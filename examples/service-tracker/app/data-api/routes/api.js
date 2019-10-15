const applicationInsights = require('applicationinsights');
const async = require('async');
const dayjs = require('dayjs');
const express = require('express');
const jsonResponse = require('../models/express/jsonResponse');
const mongoose = require('mongoose');
const path = require('path');
const relativeTime = require('dayjs/plugin/relativeTime');
const router = express.Router();
const st = require('../models/util/status');
const site = require('../models/util/site');

dayjs.extend(relativeTime);

/* Models and Telemetry event info */
const Flights = mongoose.model('Flights');
const LatestFlight = mongoose.model('LatestFlight');
const Quakes = mongoose.model('Quakes');
const LatestQuake = mongoose.model('LatestQuake');
const Weather = mongoose.model('Weather');
const LatestWeather = mongoose.model('LatestWeather');

/**
 *
 * Incorporate telemetry with App Insights
 *
 **/
var telemetry = applicationInsights.defaultClient;

const routename = path
  .basename(__filename)
  .replace('.js', ' default endpoint for ' + site.name);

/* GET JSON :: Route Base Endpoint */
router.get('/', (req, res, next) => {
  jsonResponse.json(res, routename, st.OK.code, {});
});

router.get('/status', (req, res, next) => {
  jsonResponse.json(res, routename, st.OK.code, {
    uptime: dayjs(global.start).from(
      dayjs(Math.floor(process.uptime()) * 1000 + global.start),
      true
    )
  });
});

router.get('/get/flights/:timestamp', (req, res, next) => {
  getDataObjFromDb(Flights, req.params.timestamp, (err, result) => {
    jsonResponse.json(res, 'success', st.OK.code, result);
  });
});

router.get('/get/quakes/:timestamp', (req, res, next) => {
  getDataObjFromDb(Quakes, req.params.timestamp, (err, result) => {
    jsonResponse.json(res, 'success', st.OK.code, result);
  });
});

router.get('/get/weather/:timestamp', (req, res, next) => {
  getDataObjFromDb(Weather, req.params.timestamp, (err, data) => {
    if (err) {
      jsonResponse.json(res, st.ERR.msg, st.ERR.code, err);
    } else {
      if (data.length > 0) {
        jsonResponse.json(res, st.OK.msg, st.OK.code, data);
      } else {
        jsonResponse.json(res, st.EMPTY.msg, st.EMPTY.code, data);
      }
    }
  });
});

router.get('/get/latest/:datatype', (req, res, next) => {
  getLatestObjFromDb(determineObj(req.params.datatype), (err, data) => {
    if (err) {
      jsonResponse.json(res, st.ERR.msg, st.ERR.code, err);
    } else {
      if (data.length > 0) {
        jsonResponse.json(res, st.OK.msg, st.OK.code, data);
      } else {
        res.status(204).end();
      }
    }
  });
});

router.post('/save/flights/:timestamp', (req, res, next) => {
  var latest = new LatestFlight({ Timestamp: req.params.timestamp });
  var flights = new Flights({
    Timestamp: req.params.timestamp,
    FeatureCollection: req.body
  });

  async.waterfall(
    [
      cb => {
        saveDataObjToDb(flights, (e, r) => {
          if (r) {
            cb(null, {
              FlightCount: flights.FeatureCollection.length,
              Timestamp: flights.Timestamp
            });
          }
        });
      },
      (flightDetail, cb) => {
        saveDataObjToDb(latest, (e, r) => {
          cb(e, flightDetail);
        });
      }
    ],
    (err, result) => {
      jsonResponse.json(res, 'success', st.OK.code, result);
    }
  );
});

router.post('/save/quakes/:timestamp', (req, res, next) => {
  var latest = new LatestQuake({ Timestamp: req.params.timestamp });
  var quakes = new Quakes({
    Timestamp: req.params.timestamp,
    FeatureCollection: req.body
  });

  async.waterfall(
    [
      cb => {
        saveDataObjToDb(quakes, (e, r) => {
          if (r) {
            cb(null, {
              QuakeCount: quakes.FeatureCollection.length,
              Timestamp: quakes.Timestamp
            });
          }
        });
      },
      (quakeDetail, cb) => {
        saveDataObjToDb(latest, (e, r) => {
          cb(e, quakeDetail);
        });
      }
    ],
    (err, result) => {
      jsonResponse.json(res, 'success', st.OK.code, result);
    }
  );
});

router.post('/save/weather/:timestamp', (req, res, next) => {
  var latest = new LatestWeather({ Timestamp: req.params.timestamp });
  var weather = new Weather({
    Timestamp: req.params.timestamp,
    FeatureCollection: req.body
  });

  async.waterfall(
    [
      cb => {
        saveDataObjToDb(weather, (e, r) => {
          if (r) {
            cb(null, {
              WeatherLayerCount: weather.FeatureCollection.length,
              Timestamp: weather.Timestamp
            });
          }
        });
      },
      (weatherDetail, cb) => {
        saveDataObjToDb(latest, (e, r) => {
          cb(e, weatherDetail);
        });
      }
    ],
    (err, result) => {
      jsonResponse.json(res, 'success', st.OK.code, result);
    }
  );
});

function saveDataObjToDb(data, cb) {
  data
    .save()
    .then(doc => {
      cb(null, true);
    })
    .catch(err => {
      if (err)
        handleError(site.name + ' func - saveDataObjToDb :: error saving data');
      cb(err, false);
    });
}

function getDataObjFromDb(obj, timestamp, cb) {
  obj
    .findOne({ Timestamp: timestamp })
    .limit(1)
    .exec((err, doc) => {
      if (err)
        handleError(
          site.name + ' func - getDataObjFromDb :: error retrieving data'
        );
      cb(err, doc);
    });
}

function determineObj(objName) {
  switch (objName) {
    case 'flights':
      return LatestFlight;
    case 'quakes':
      return LatestQuake;
    case 'weather':
      return LatestWeather;
    default:
      break;
  }
}

// function getQuakesFromDb(timestamp, cb){
//     Quakes
//         .findOne({Timestamp: timestamp})
//         .limit(1)
//         .exec( (err, doc) => {
//             cb(err, doc)
//         })
// }

// function getWeatherFromDb(timestamp, cb){
//     Weather
//         .findOne({Timestamp: timestamp})
//         .limit(1)
//         .exec( (err, doc) => {
//             cb(err, doc)
//         })
// }

function getLatestObjFromDb(obj, cb) {
  obj
    .find()
    .sort({ Timestamp: -1 })
    .limit(1)
    .exec((err, doc) => {
      if (err)
        handleError(
          site.name + ' func - getLatestObjFromDb :: error retrieving data'
        );
      cb(err, doc);
    });
}

function handleError(message) {
  console.log(message);
}

module.exports = router;
