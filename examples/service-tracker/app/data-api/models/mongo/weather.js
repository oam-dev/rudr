const mongoose = require('mongoose');

var Schema = mongoose.Schema;

var weatherSchema = new Schema({
  Timestamp: String,
  FeatureCollection: mongoose.Schema.Types.Mixed
});

mongoose.model('Weather', weatherSchema, 'Weather');
