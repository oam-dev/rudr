const webpack = require('webpack');
const rp = require('request-promise');
const async = require('async');

module.exports = {
  configureWebpack: {
    plugins: [
      new webpack.DefinePlugin({
        'process.env': {
          FLIGHT_API: JSON.stringify(process.env.FLIGHT_API_ROOT),
          WEATHER_API: JSON.stringify(process.env.WEATHER_API_ROOT),
          QUAKES_API: JSON.stringify(process.env.QUAKES_API_ROOT)
        }
      })
    ]
  },
  lintOnSave: false,
  devServer: {
    disableHostCheck: true,
    host: '0.0.0.0',
    port: 8080,
    // show variables when running http://localhost:8080/variables
    before: function(app) {
      app.get('/variables', (req, res) => {
        var currentEnv = {
          quakes: process.env.QUAKES_API_ROOT,
          weather: process.env.WEATHER_API_ROOT,
          flights: process.env.FLIGHT_API_ROOT
        };
        res.json({ custom: currentEnv });
      });
    },
    proxy: {
      '/api/flights/current': {
        target: process.env.FLIGHT_API_ROOT + 'latest',
        changeOrigin: true,
        pathRewrite: {
          '^/api/flights/current': ''
        }
      },
      '/api/flights/status': {
        target: process.env.FLIGHT_API_ROOT + 'status',
        changeOrigin: true,
        pathRewrite: {
          '^/api/flights/status': ''
        }
      },
      '/api/flights/refresh': {
        target: process.env.FLIGHT_API_ROOT + 'refresh',
        changeOrigin: true,
        pathRewrite: {
          '^/api/flights/refresh': ''
        }
      },
      '/api/weather/current': {
        target: process.env.WEATHER_API_ROOT + 'latest',
        changeOrigin: true,
        pathRewrite: {
          '^/api/weather/current': ''
        }
      },
      '/api/weather/status': {
        target: process.env.WEATHER_API_ROOT + 'status',
        changeOrigin: true,
        pathRewrite: {
          '^/api/weather/status': ''
        }
      },
      '/api/weather/refresh': {
        target: process.env.WEATHER_API_ROOT + 'refresh',
        changeOrigin: true,
        pathRewrite: {
          '^/api/weather/refresh': ''
        }
      },
      '/api/quakes/current': {
        target: process.env.QUAKES_API_ROOT + 'latest',
        changeOrigin: true,
        pathRewrite: {
          '^/api/quakes/current': ''
        }
      },
      '/api/quakes/status': {
        target: process.env.QUAKES_API_ROOT + 'status',
        changeOrigin: true,
        pathRewrite: {
          '^/api/quakes/status': ''
        }
      },
      '/api/quakes/refresh': {
        target: process.env.QUAKES_API_ROOT + 'refresh',
        changeOrigin: true,
        pathRewrite: {
          '^/api/quakes/refresh': ''
        }
      },
      '/api/k8s/nodes': {
        target: 'http://localhost:3000/k8s-service/pods', // NEED TO INCORPORATE THIS
        changeOrigin: true,
        pathRewrite: {
          '^/api/k8s/nodes': ''
        }
      },
      '/api/quakes/stats': {
        target: 'http://localhost:3000/api/stats', //NEED TO CHANGE + ADD THIS PORT
        changeOrigin: true,
        pathRewrite: {
          '^/api/quakes/stats': ''
        }
      }
    }
  }
};
