const mongoose = require('mongoose');

const Schema = mongoose.Schema;

const latestSchema = new Schema({
  Timestamp: String,
  Created: { type: Date, default: Date.now }
});

mongoose.model('LatestQuake', latestSchema, 'LatestQuake');
