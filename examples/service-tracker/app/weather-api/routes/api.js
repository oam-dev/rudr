var applicationInsights = require('applicationinsights'),
    async = require('async'),
    cacheServiceUri = process.env.CACHE_SERVICE_URI,
    dataServiceUri = process.env.DATA_SERVICE_URI,
    dayjs = require('dayjs'),
    express = require('express'),
    fs = require('fs'),
    jsonResponse = require('../models/express/jsonResponse'),
    path = require('path'),
    router = express.Router(),
    relativeTime = require('dayjs/plugin/relativeTime'),
    rp = require('request-promise'),
    st = require('../models/util/status'),
    site = require('../models/util/site')

    dayjs.extend(relativeTime)
    

/**
 * 
 * Incorporate telemetry with App Insights
 * 
 **/
var telemetry = applicationInsights.defaultClient

const routename = path.basename(__filename).replace('.js', ' default endpoint for ' + site.name)

const weatherIconBaseUrl = 'https://developer.accuweather.com/sites/default/files/'

/**
 * 
 * HTTP GET /
 * default endpoint
 * 
 **/
router.get('/', (req, res, next) => {
    jsonResponse.json( res, routename, st.OK.code, {} )
})

/** 
 * 
 * HTTP GET /status
 * JSON
 * ENDPOINT FOR DASHBOARD SERVICE STATUS
 * 
 **/
router.get('/status', (req, res, next) => {
    async.waterfall([
        (cb) => {
            getFromDataApi('get/latest/weather', (e, d) => {
                if(e){
                    handleError(site.name +'/status :: error retrieving data')
                    cb(e, null)
                }
                else{
                    if(d.payload!=null){
                        cb(null, d.payload[0].Timestamp)
                    }else{
                        handleError(site.name +'/status :: no data in cosmosdb')
                        cb(site.ERR_NO_DATA, null)
                    }
                }  
            })
        }
    ],(e,r) => {
        if(e){
            if (e === site.ERR_NO_DATA){
                res.status(204).end()
            } else {
                jsonResponse.json(res, st.ERR.msg, st.ERR.code, e)
            }
        }else{
            jsonResponse.json( res, routename, st.OK.code, {
                uptime: dayjs(global.start).from(dayjs((Math.floor(process.uptime())*1000) + global.start), true),
                latest:dayjs(Number(r)).format('MM/DD/YYYY h:mm A')
            })
        }
    })

})

/**
 * 
 * HTTP GET /latest
 * JSON
 * USES DATABASE
 * NO CACHE
 * 
 **/
router.get('/latest', (req, res, next) => {

    async.waterfall([
        (cb) => {
            getFromDataApi('get/latest/weather', (e, d) => {
                if(e){
                    handleError(site.name +'/latest :: error retrieving latest timestamp')
                    cb(e, null)
                }else{
                    cb(null, d.payload[0].Timestamp)
                }
            })
        },
        (timestamp, cb) => {
            getFromDataApi('get/weather/' + timestamp, (e, d) => {
                if(e){
                    handleError(site.name +'/latest :: error retrieving weather with timestamp')
                    cb(e, null)
                }else{
                    cb(null, d.payload.FeatureCollection)
                }
            })

        }
    ],(e,r) => {
        if(e) {
            jsonResponse.json( res, st.ERR.msg, st.ERR.code, e )
        }
        jsonResponse.json( res, st.OK.msg, st.OK.code, r)
    })

})

/**
 * 
 * HTTP GET /refresh
 * JSON
 * API CALL TO ACCUWEATHER
 * SAVE TO DATABASE
 * NO CACHE
 * 
 **/
router.get('/refresh', (req, res, next) => {

    async.waterfall([
        (cb) => {
            getWeatherTopCities(150, (e,d) => {
                if(e) cb(e, null)
                cb(null, d)
            })
        },
        (data, cb) => {
            buildGeoJson(data, (e,d) => {
                if(e) cb(e, null)
                cb(null, d, dayjs().valueOf())
            })
        },
        (data, key, cb) => {
            saveToDataApi(key, data, (e,r) => { 
                if(e) cb(e, null)
                cb(null, r)
            } )
        }
    ],(e,r) => {
        if(e){
            res.status(500).end()
            next()
        }
        jsonResponse.json( res, st.OK.msg, st.OK.code, r)
    })

})


router.get('/cityPositions', (req, res, next) => {
    var weatherLocales = []
    getWeatherCities((err, data) =>{
        async.each(data, (locale, callback) => {
            var lat = locale.geometry.coordinates[1]
            var long = locale.geometry.coordinates[0]
            getGeoPositionKey(lat, long, (e, d) => {
                locale.properties['AWPositionKey'] = d.Key
                weatherLocales.push(locale)
                callback()
            })
        }, (err) => {
            if(err){
                console.log(err)
                jsonResponse.json( res, st.ERR.msg, st.ERR.code, err)
            }else{
                console.log('all locales processed successfully')
                jsonResponse.json( res, st.OK.msg, st.OK.code, weatherLocales)
            }
        })
    })
})


function getWeatherCities(cb) {
    console.log(weatherCities1000.length)

    // top 50 us cities by population
    var nationalWeatherCities = []
    async.each(weatherCities1000, (city, callback) => {

        /* create the GeoJSON feature for this city */
          nationalWeatherCities.push({ 
            type: 'Feature',
            properties: {
              Name:city.city,
              Population: city.population,
              Icon:'https://developer.accuweather.com/sites/default/files/01-s.png',
              Condition:'',
              Temperature:0
            },
            geometry: { 
              type: 'Point',
              coordinates:[ city.longitude, city.latitude ]
            }
          })
          callback()

    }, (err) => {
        if(err){
            console.log(err)
            cb(err, null)
        }else{
            console.log('all cities processed successfully')
            cb(null, nationalWeatherCities)
        }
    })
}

function getWeatherTopCities(count, cb){

    var opt = { uri: 'http://dataservice.accuweather.com/currentconditions/v1/topcities/' + count + '?apikey=lfl6t1f1pQQ87ZMA8FdjRTemDJtgeiYe', json: true }
    
    try {
        rp(opt)
        .then(data => {
            cb(null, data)
        })
        .catch(err => {
            cb(err, null)
        })
    }
    catch (error) {
        cb(error, null)
    }
    
}

function buildGeoJson(data, cb){

    var geoJsonArray = []

    var layerBlue = {id: 'mapblue', textColor:'#37EEFF', features:[]},
        layerYellow = {id: 'mapyellow', textColor:'#ffee38', features:[]},
        layerYellowOrange = {id: 'mapyelloworange', textColor:'#ffc038', features:[]},
        layerOrange = {id: 'maporange', textColor:'#ff8138', features:[]},
        layerRed = {id: 'mapred', textColor:'#ff6338', features:[]},
        layerBrightRed = {id: 'mapbrightred', textColor:'#ff0000', features:[]}


    async.each(data, (city, callback) => {
        var weatherIcon = weatherIconBaseUrl.concat(city.WeatherIcon, '-s.png')
        if (city.WeatherIcon < 10) weatherIcon = weatherIconBaseUrl.concat('0', city.WeatherIcon, '-s.png')
        
        var feature = { 
            type: 'Feature',
            properties: {
              Name: city.EnglishName,
              Country: city.Country.EnglishName,
              Icon: weatherIcon,
              Condition: city.WeatherText,
              Temperature: city.Temperature.Imperial.Value
            },
            geometry: { 
              type: 'Point',
              coordinates:[ city.GeoPosition.Longitude, city.GeoPosition.Latitude ]
            }
        }
        
        feature.properties["Temperature"] = feature.properties["Temperature"].toString() + '°'

        if (city.Temperature.Imperial.Value  <= 65){
            layerBlue.features.push(feature)
        }
        if (city.Temperature.Imperial.Value > 65 && city.Temperature.Imperial.Value <= 72) {
            layerYellow.features.push(feature)
        }
        if (city.Temperature.Imperial.Value > 72 && city.Temperature.Imperial.Value <= 79) {
            layerYellowOrange.features.push(feature)
        }
        if (city.Temperature.Imperial.Value > 79 && city.Temperature.Imperial.Value <= 88) {
            layerOrange.features.push(feature)
        }
        if (city.Temperature.Imperial.Value > 88 && city.Temperature.Imperial.Value <= 97) {
            layerRed.features.push(feature)
        }
        if (city.Temperature.Imperial.Value > 97){
            layerBrightRed.features.push(feature)
        }
        
        callback()

    }, (err) => {
        if(err){
            cb(err, null)
        }else{
            geoJsonArray.push(layerBlue, layerYellow, layerYellowOrange, layerOrange, layerRed, layerBrightRed)
            cb(null, geoJsonArray)
        }
    })
}

function getGeoPositionKey(lat,long, cb){

    // &q=40.7127837%2C-74.0059413&toplevel=true
    var query = "&q=" + lat + "%2C" + long + "&toplevel=true"

    var url = accuweatherBaseUrl.concat(positionSearchPath, accuweatherApiKey, query)
    console.log(url)
    var opt = { 
        uri: url,
        headers: { 'User-Agent': 'Request-Promise' },
        json: true
    }

    rp(opt)
      .then(out => {
          console.log(out)
        cb(null, out)
    })
    .catch(err => {
        console.log(err)
        cb(err, null)
    })


    //http://dataservice.accuweather.com/locations/v1/cities/geoposition/search?apikey=V7jqpMmCwAZMLQfrEGmMKz5oSepCeeh8&q=40.7127837%2C-74.0059413&toplevel=true
}

function getConditionsForKey(key, cb){
    //http://dataservice.accuweather.com/currentconditions/v1/347625?apikey=lfl6t1f1pQQ87ZMA8FdjRTemDJtgeiYe
    // &q=40.7127837%2C-74.0059413&toplevel=true
    var query = key +'?' + accuweatherApiKey

    var url = accuweatherBaseUrl.concat(conditionSearchPath, query)
    console.log(url)
    var opt = { 
        uri: url,
        headers: { 'User-Agent': 'Request-Promise' },
        json: true
    }

    rp(opt)
      .then(out => {
        //console.log(out)
        cb(null, out)
    })
    .catch(err => {
        //console.log(err)
        cb(err, null)
    })


    //http://dataservice.accuweather.com/locations/v1/cities/geoposition/search?apikey=V7jqpMmCwAZMLQfrEGmMKz5oSepCeeh8&q=40.7127837%2C-74.0059413&toplevel=true
}

function postCacheItem(key, data, event, cb){
    // telemetry.trackEvent({name: event})
    var url = cacheServiceUri + 'set/' + key
    var obj = JSON.stringify(data);
    console.log(url)
    console.log(obj)
    var opt = { method: 'POST',
        uri: url,
        headers: { 'User-Agent': 'Request-Promise' },
        body: obj,
        json: true
    }

    rp(opt)
      .then(out => {
        cb(null, out)
    })
    .catch(err => {
        cb(err, null)
    })
}

function saveToDataApi(timestamp, data, cb) {
    var url = dataServiceUri + 'save/weather/' + timestamp
    
    var opt = { method: 'POST',
        uri: url,
        body: data,
        json: true
    }

    rp(opt)
      .then(out => {
        cb(null, out)
    })
    .catch(err => {
        handleError(site.name +' func - saveToDataApi :: error saving flights to DB:')
        cb(err, null)
    })
}

/* DB API GET CALL */
function getFromDataApi(path, cb){
    
    var url = dataServiceUri + path
    
    var opt = { uri: url, json: true, resolveWithFullResponse: true  }

    rp(opt)
    .then( out => {
        if( out.statusCode === 200){
            cb(null, out.body)
        }
        if( out.statusCode === 204){
            cb(null, {payload:null})
        }
        
    })
    .catch( err => {
        handleError(site.name +' func - getFromDataApi :: error retrieving data' + err)
        cb(err, null)
    })

}



function handleError(message) {
    console.log(message)
}


module.exports = router