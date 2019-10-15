var bodyParser = require('body-parser'),
    createError = require('http-errors'),
    dayjs = require('dayjs'),
    express = require('express'),
    logger = require('morgan'),
    path = require('path'),
    relativeTime = require('dayjs/plugin/relativeTime')

    dayjs.extend(relativeTime)

    global.start = dayjs().valueOf()
    
if (process.env.NODE_ENV != 'container') {
  require('dotenv').config({path: path.join(__dirname, '.env.local')})
}

var apiRouter = require('./routes/api')

var app = express()
app.set('etag', 'strong');
app.use(logger('dev'))
app.use(bodyParser.json({limit:'2mb'}))
app.use('/', apiRouter)

app.use(function(req, res, next) {
  next(createError(404))
})

app.use(function(req, res, next) {
  
  res.setHeader('Access-Control-Allow-Origin', '*')
  res.setHeader(
    'Access-Control-Allow-Methods',
    'GET, POST, OPTIONS, PUT, PATCH, DELETE'
  )
  res.setHeader(
    'Access-Control-Allow-Headers',
    'X-Requested-With,content-type'
  )

  res.append('Last-Modified', (new Date()).toUTCString())

  next()
})

app.use(function(err, req, res, next) {
  res.locals.message = err.message
  res.locals.error = req.app.get('env') === 'development' ? err : {}
  res.status(err.status || 500)
  res.send(err)
})

module.exports = app
