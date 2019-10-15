const mongoose = require('mongoose');

var Schema = mongoose.Schema;

var quakeSchema = new Schema({
  Timestamp: String,
  FeatureCollection: mongoose.Schema.Types.Mixed
});

mongoose.model('Quakes', quakeSchema, 'Quakes');
