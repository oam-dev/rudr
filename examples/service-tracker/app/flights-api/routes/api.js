const applicationInsights = require('applicationinsights');
const async = require('async');
const cacheServiceUri = process.env.CACHE_SERVICE_URI;
const dataServiceUri = process.env.DATA_SERVICE_URI;
const dayjs = require('dayjs');
const express = require('express');
const jsonResponse = require('../models/express/jsonResponse');
const path = require('path');
const router = express.Router();
const relativeTime = require('dayjs/plugin/relativeTime');
const rp = require('request-promise');
const st = require('../models/util/status');
const site = require('../models/util/site');

/**
 *
 * Incorporate telemetry with App Insights
 *
 **/

let telemetry = applicationInsights.defaultClient;

const routename = path
  .basename(__filename)
  .replace('.js', ' default endpoint for ' + site.name);

/**
 *
 * HTTP GET /
 * default endpoint
 *
 **/
router.get('/', (req, res, next) => {
  jsonResponse.json(res, routename, st.OK.code, {});
});

/**
 *
 * HTTP GET /status
 * JSON
 * ENDPOINT FOR DASHBOARD SERVICE STATUS
 *
 **/
router.get('/status', (req, res, next) => {
  async.waterfall(
    [
      cb => {
        getFromDataApi('get/latest/flights', (e, d) => {
          if (e) {
            handleError(site.name + '/status :: error retrieving data');
            cb(e, null);
          } else {
            if (d.payload != null) {
              cb(null, d.payload[0].Timestamp);
            } else {
              handleError(site.name + '/status :: no data in cosmosdb');
              cb(site.ERR_NO_DATA, null);
            }
          }
        });
      }
    ],
    (e, r) => {
      if (e) {
        if (e === site.ERR_NO_DATA) {
          res.status(204).end();
        } else {
          jsonResponse.json(res, st.ERR.msg, st.ERR.code, e);
        }
      } else {
        jsonResponse.json(res, routename, st.OK.code, {
          uptime: dayjs(global.start).from(
            dayjs(Math.floor(process.uptime()) * 1000 + global.start),
            true
          ),
          latest: dayjs(Number(r)).format('MM/DD/YYYY h:mm A')
        });
      }
    }
  );
});

/**
 *
 * HTTP GET /latest
 * JSON
 * USES DATABASE
 * NO CACHE
 *
 **/
router.get('/latest', (req, res, next) => {
  async.waterfall(
    [
      cb => {
        getFromDataApi('get/latest/flights', (e, d) => {
          if (e) {
            handleError(
              site.name + '/latest :: error retrieving latest timestamp'
            );
            cb(e, null);
          } else {
            cb(null, d.payload[0].Timestamp);
          }
        });
      },
      (timestamp, cb) => {
        getFromDataApi('get/flights/' + timestamp, (e, d) => {
          if (e) {
            handleError(
              site.name + '/latest :: error retrieving flights with timestamp'
            );
            cb(e, null);
          } else {
            cb(null, d.payload.FeatureCollection);
          }
        });
      }
    ],
    (e, r) => {
      if (e) {
        jsonResponse.json(res, st.ERR.msg, st.ERR.code, e);
      }
      jsonResponse.json(res, st.OK.msg, st.OK.code, r);
    }
  );
});

/**
 *
 * HTTP GET /refresh
 * JSON
 * API CALL TO OPENSKY
 * SAVE TO DATABASE
 * NO CACHE
 *
 **/
router.get('/refresh', (req, res, next) => {
  var querypath = 'all';

  async.waterfall(
    [
      cb => {
        getFlightData(querypath, 'refreshflightdata', (err, data) => {
          if (err) {
            handleError(site.name + '/refresh :: error retrieving data: ');
          } else {
            cb(null, data);
          }
        });
      },
      (data, cb) => {
        cb(null, data, dayjs().valueOf());
      },
      (data, timestamp, cb) => {
        buildGeoJson(data.states, (err, result) => {
          cb(null, result, timestamp);
        });
      },
      (data, key, cb) => {
        saveToDataApi(key, data, (e, r) => {
          cb(null, r);
        });
      }
    ],
    (e, r) => {
      if (e) {
        jsonResponse.json(res, st.ERR.msg, st.ERR.code, e);
      } else {
        jsonResponse.json(res, st.OK.msg, st.OK.code, r);
      }
    }
  );
});

/**
 *
 * HTTP GET /current
 * JSON
 * NO CACHE
 * NO DATABASE
 *
 **/
router.get('/current', (req, res, next) => {
  var querypath = 'all';
  var event = 'no_cache';
  getFlightData(querypath, event, (err, data) => {
    if (err) {
      jsonResponse.json(res, st.ERR.msg, st.ERR.code, err);
      next();
    }

    buildGeoJson(data.states, (fmtError, flights) => {
      jsonResponse.json(res, st.OK.msg, st.OK.code, flights);
    });
  });
});

/* OPENSKY API */
function getFlightData(querypath, event, cb) {
  // telemetry.trackEvent({name: event})
  var opt = {
    uri: 'https://opensky-network.org/api/states/' + querypath,
    json: true
  };

  rp(opt)
    .then(data => {
      cb(null, data);
    })
    .catch(err => {
      handleError(
        site.name +
          ' func - getFlightData :: error retrieving flights from opensky'
      );
      cb(err, null);
    });
}

/* CACHE API SET CALL */
function postCacheItem(key, data, event, cb) {
  // telemetry.trackEvent({name: event})
  var url = cacheServiceUri + 'set/' + key;
  var obj = JSON.stringify(data);
  var opt = {
    method: 'POST',
    uri: url,
    headers: { 'User-Agent': 'Request-Promise' },
    body: obj,
    json: true
  };

  rp(opt)
    .then(out => {
      cb(null, out);
    })
    .catch(err => {
      cb(err, null);
    });
}

/* CACHE API GET CALL */
function getCacheItem(key, cb) {
  var opt = {
    uri: cacheServiceUri + key,
    headers: { 'User-Agent': 'Request-Promise' },
    json: true
  };
  rp(opt)
    .then(data => {
      cb(null, data);
    })
    .catch(err => {
      cb(err, null);
    });
}

/* DB API SAVE CALL */
function saveToDataApi(timestamp, data, cb) {
  var url = dataServiceUri + 'save/flights/' + timestamp;

  var opt = {
    method: 'POST',
    uri: url,
    body: data,
    json: true
  };

  rp(opt)
    .then(out => {
      cb(null, out);
    })
    .catch(err => {
      handleError(
        site.name + ' func - saveToDataApi :: error saving flights to DB:'
      );
      cb(err, null);
    });
}

/* DB API GET CALL */
function getFromDataApi(path, cb) {
  var url = dataServiceUri + path;

  var opt = { uri: url, json: true, resolveWithFullResponse: true };

  rp(opt)
    .then(out => {
      if (out.statusCode === 200) {
        cb(null, out.body);
      }
      if (out.statusCode === 204) {
        cb(null, { payload: null });
      }
    })
    .catch(err => {
      handleError(
        site.name + ' func - getFromDataApi :: error retrieving data' + err
      );
      cb(err, null);
    });
}

/* BUILD THE GEOJSON ELEMENTS FROM FLIGHTS */
function buildGeoJson(flights, cb) {
  var flightGeoJson = [];
  var includedCountries = ['United States', 'Canada', 'Mexico'];
  async.each(
    flights,
    (flight, callback) => {
      if (
        flight[8] ||
        flight[7] <= 0 ||
        flight[5] === null ||
        flight[1].toString().replace(/ /g, '') === '' ||
        flight[1].length <= 6 ||
        includedCountries.indexOf(flight[2]) === -1
      ) {
        callback();
      } else {
        /* create the GeoJSON feature for this flight */
        var feature = {
          type: 'Feature',
          properties: {
            FlightNumber: flight[1].toString().replace(/ /g, ''),
            Country: flight[2],
            Altitude: flight[7],
            AirSpeed: flight[9],
            Heading: flight[10]
          },
          geometry: {
            type: 'Point',
            coordinates: [flight[5], flight[6]]
          }
        };

        /* Add this flights GeoJSON to the array */
        flightGeoJson.push(feature);
        callback();
      }
    },
    err => {
      if (err) {
        cb(err, null);
      } else {
        cb(null, flightGeoJson);
      }
    }
  );
}

function handleError(message) {
  console.log(message);
}

module.exports = router;
