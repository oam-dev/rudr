/**
 * JSON Response for Express Web API's
 * @param {object} res - Express Response Object
 * @param {any} msg - Response message as property in object
 * @param {number} status - HTTP Status code
 * @param {object} payload - JSON payload object
 */

module.exports.json =  (res, msg, status, payload) => {
    
    res.json( { message: msg, payload: payload } ).status( status )

}