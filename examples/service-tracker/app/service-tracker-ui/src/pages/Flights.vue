<template>
  <section>
    <div class="row">
      <div class="col-md-6 col-xl-3">
        <h6>FLIGHT MAP</h6>
      </div>
    </div>
    <div v-bind:style="{ height: mapHeight + 'px'}">
        <div id='map'></div>
    </div>
  </section>
</template>
<script>

/* eslint-disable */
import mapboxgl from 'mapbox-gl/dist/mapbox-gl'

let map
let vm

export default {
  data () {
    return {
      mapHeight:'100'
    }
  },
  created() {
    this.mapHeight = (window.innerHeight - 240)
  },
  mounted() {
    this.$nextTick(function() {
      window.addEventListener('resize', this.getWindowHeight);
      //Init
      this.getWindowHeight()
    })


    vm = this
    mapboxgl.accessToken =
      'pk.eyJ1Ijoic29ub2pvcmVsIiwiYSI6ImNqaDl1Z25udzAybGMzNnBmbzl4NDBsam0ifQ.itgTNw7IhsoPTwkxiPz7Vw';
    map = new mapboxgl.Map({
      container: 'map',
      style: 'mapbox://styles/sonojorel/cjhw8422i15g72snx2zsof3s1',
      center: [-79.995888, 40.440624],
      zoom: 2
    })

    map.on('load', function() {
      vm.loadFlights()
    })

    //** show current map center lat lng on page **//
    map.on('moveend', function(e) {
      vm.latitude = map.getCenter().lat
      vm.longitude = map.getCenter().lng
    })

  },
  methods: {
    addLayer(obj){
      map.addLayer(
          {
            'id': 'flights',
            'type': 'symbol',
            'source': {
              'type': 'geojson',
              'data': {
                  'type': 'FeatureCollection',
                  'features': obj
              }
            },
            'layout': {
              'icon-image': 'green-plane',
              'icon-size': .8,
              //'text-field': '{mag}',
              'icon-rotate': {
                  'type': 'identity',
                  'property': 'Heading'
              },
              'icon-allow-overlap': true,
              'text-allow-overlap':true,
              // 'text-font': ['Open Sans Semibold', 'Arial Unicode MS Bold'],
              //'text-size': 10,
              //'text-offset': [0, 0.6],
              //'text-anchor': 'top'
            },
            'paint':{
              // 'text-color':'#555',
              // 'text-halo-color':'rgba(255, 255, 255, .50)',
              // 'text-halo-width':1
            }
            })

      map.on('click', 'flights', function (e) {

        var pinDetail = e.features[0].properties
        var popHtml = document.getElementById('popUpHtml').innerHTML
        var popInfo = document.getElementById('popUpDetailHtml').innerHTML
        popHtml = popHtml.replace('##title##', pinDetail.FlightNumber).replace('##subtitle##', 'FLIGHT NUMBER')
        var detailHtml = popInfo.replace('##name##','HEADING').replace('##value##', pinDetail.Heading + ' DEGREES')
        detailHtml = detailHtml.concat(popInfo.replace('##name##','ALTITUDE').replace('##value##', ((Math.round((pinDetail.Altitude*3.2808))*10)/10).toString() + ' FEET'))
        detailHtml = detailHtml.concat(popInfo.replace('##name##','AIR SPEED').replace('##value##', Math.round(pinDetail.AirSpeed * 3600 / 1610.3).toString() + ' MPH'))
        popHtml = popHtml.replace('##info##', detailHtml)

        var coordinates = e.features[0].geometry.coordinates.slice()
        

        while (Math.abs(e.lngLat.lng - coordinates[0]) > 180) {
            coordinates[0] += e.lngLat.lng > coordinates[0] ? 360 : -360
        }

        var popup = new mapboxgl.Popup()
            .setLngLat(coordinates)
            .setHTML(popHtml)
            .addTo(map)

        document.querySelector("i.closePop").addEventListener("click", () => {
          popup.remove()
        })
        
        map.flyTo({center: e.features[0].geometry.coordinates, zoom: 9, speed: 0.75, curve: 1})
    })

    },
    loadFlights() {
      let payload

      const myRequest = new Request('/api/flights/current')

      fetch(myRequest)
      .then((response) => { 
        return response.json() })
      .then((data) => {
        payload = data.payload
        vm.addLayer(payload)
      })
       
    },
    getWindowHeight(event) {
        this.mapHeight = (window.innerHeight - 240)
      }
  }
}
</script>
<style lang='scss'>



#map {
  width: 100%;
  // min-height: 880px;
  min-height: 100%;
  height:auto !important; /* cross-browser */
  height: 100%; /* cross-browser */
}

.mapboxgl-map {
  font-size: 14px;
  font-family: 'Muli', Arial, sans-serif;
}
.mapboxgl-popup{
  min-width: 150px;
}
.mapboxgl-popup-content{
  background: #06204d;
  box-shadow: none;
  padding: 8px 2px 0px 2px !important;
}
.mapboxgl-popup-content * .card {
  background: none !important;
}
.card {
  background: none !important;
  box-shadow: none !important;
  margin-bottom: 2px !important;
  color: #8bb837 !important;
}

.card .card-body {
    padding: 5px 24px 5px 5px !important;
}
h5.card-title{
  font-family: 'Montserrat', sans-serif !important;
  font-size: 22px!important;
  color:#8eb9ee !important;
  font-weight: 700 !important;
  padding: 4px 15px 0px 0px !important;
}
ul.list-group{
  background: #FFF !important;
}
li.list-group-item{
  padding: 4px 10px !important;
  background: none !important;
  border: none !important;
}
.mapboxgl-popup-tip {
  border-bottom-color: #06204d !important;
  border-top-color: #06204d !important;
}
.mapboxgl-popup-close-button {
    position: absolute;
    right: 0;
    top: 0;
    border: 0;
    border-radius: 0 0 0 0;
    color:#FFF;
    font-size: 18px;
    font-variant: small-caps;
    cursor: pointer;
    background-color: transparent;
}
.mapboxgl-popup-close-button{
  display: none;
}

.closePop{
  position: absolute;
  font-size:11px !important;
  top: -6px;
  right: 2px;
  color:#8bb837;
  cursor: pointer;
}

.blockquote-footer::before{
  content: '' !important;
}
.blockquote-footer{
  margin-bottom: -8px;
  display: block;
  font-size: 9px;
  color: #999;
}
.popValue{
  font-family: 'Montserrat', sans-serif !important;
  font-size: 14px;
  font-weight: 700;
}
.blockquote-footer.popTitleHead{
  color:#ACACAC;
}
.blockquote-footer .popInfoHead{
  color:#555;
  text-transform: uppercase;
  font-size: 9px;
}

</style>
