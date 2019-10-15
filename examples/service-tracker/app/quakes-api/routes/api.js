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
var telemetry = applicationInsights.defaultClient;

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
        getFromDataApi('get/latest/quakes', (e, d) => {
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
 * HTTP GET /current
 * JSON
 * NO CACHE
 * NO DATABASE
 *
 **/
router.get('/current', (req, res, next) => {
  var event = 'no_cache';
  getQuakesData(event, (err, data) => {
    if (err) {
      jsonResponse.json(res, st.ERR.msg, st.ERR.code, err);
      next();
    }
    jsonResponse.json(res, st.OK.msg, st.OK.code, data);
  });
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
        getFromDataApi('get/latest/quakes', (e, d) => {
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
        getFromDataApi('get/quakes/' + timestamp, (e, d) => {
          if (e) {
            handleError(
              site.name + '/latest :: error retrieving quakes with timestamp'
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
 * API CALL TO USGS FOR FLIGHTS
 * SAVE TO DATABASE
 * NO CACHE
 *
 **/
router.get('/refresh', (req, res, next) => {
  async.waterfall(
    [
      cb => {
        getQuakesData('refreshquakesdata', (err, data) => {
          cb(null, data);
        });
      },
      (data, cb) => {
        cb(null, data, dayjs().valueOf());
      },
      (data, timestamp, cb) => {
        cb(null, data.features, timestamp);
      },
      (data, key, cb) => {
        saveToDataApi(key, data, (e, r) => {
          cb(null, r);
        });
      }
    ],
    (e, r) => {
      jsonResponse.json(res, st.OK.msg, st.OK.code, r);
    }
  );
});

/* USGS API */
function getQuakesData(event, cb) {
  console.log('rp to usgs:');

  // telemetry.trackEvent({name: event})
  var opt = {
    uri:
      'https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_month.geojson',
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

/* CACHE API SET CALL */
function postCacheItem(key, data, event, cb) {
  // telemetry.trackEvent({name: event})
  var url = cacheServiceUri + 'set/' + key;
  var obj = JSON.stringify(data);
  console.log(url);
  console.log(obj);
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
  var url = dataServiceUri + 'save/quakes/' + timestamp;

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

function handleError(message) {
  console.log(message);
}

module.exports = router;
