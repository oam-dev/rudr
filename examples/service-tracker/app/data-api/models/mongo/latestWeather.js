const mongoose = require('mongoose');

var Schema = mongoose.Schema;

var latestSchema = new Schema({
  Timestamp: String,
  Created: { type: Date, default: Date.now }
});

mongoose.model('LatestWeather', latestSchema, 'LatestWeather');
