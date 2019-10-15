const mongoose = require('mongoose');

var Schema = mongoose.Schema;

var flightSchema = new Schema({
  Timestamp: String,
  FeatureCollection: mongoose.Schema.Types.Mixed
});

mongoose.model('Flights', flightSchema, 'Flights');
