<template>
  <section>
    <div class="row">
      <div class="col-md-6 col-xl-3">
        <h6>CURRENT WEATHER CONDITIONS</h6>
      </div>
    </div>

    <div class="weatherouter" v-bind:style="{ height: mapHeight + 'px'}">
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

    map.on('load', () => { 
      vm.loadWeather()
    })

    //** show current map center lat lng on page **//
    map.on('moveend', (e) => {
      vm.latitude = map.getCenter().lat
      vm.longitude = map.getCenter().lng
    })

  },
  methods: {
    addLayer(obj){
      var objlayer = {
        'id': obj.id,
        'type': 'symbol',
        'source': {
          'type': 'geojson',
          'data': {
              'type': 'FeatureCollection',
              'features': obj.features
          }
        },
        'layout': {
          'text-field': '{Temperature}',
          'icon-allow-overlap': true,
          'text-allow-overlap': false,
          'text-font': ['League Spartan Bold','Komika Title - Axis Regular' ],
          'text-size': 26,
        },
        'paint':{
          'text-color': obj.textColor,
          'text-halo-color': '#222',
          'text-halo-width': 1.75
        }
      }
      map.addLayer(objlayer)

      map.on('click', obj.id, function (e) {
        var coordinates = e.features[0].geometry.coordinates.slice()
        
        var pinDetail = e.features[0].properties
        var popHtml = document.getElementById('popUpHtml').innerHTML
        
        popHtml = popHtml.replace('##title##', pinDetail.Temperature).replace('##subtitle##', pinDetail.Name.toUpperCase())

        var popInfo = document.getElementById('popUpDetailHtml').innerHTML
        var popInfoHtml = popInfo.replace('##name##','CONDITIONS').replace('##value##', pinDetail.Condition.toUpperCase())
        var popImage = document.getElementById('popUpDetailImage').innerHTML
        var popImageHtml = popImage.replace('##image##','<img src="' + pinDetail.Icon + '" alt="batman sucks"/>')
        var detailHtml = popInfoHtml.concat(popImageHtml)
        popHtml = popHtml.replace('##info##', detailHtml)


        // var detail = e.features[0].properties
        // var header = ('<h2>' + detail.Name + '</h2><ul>')
        // var iconInfo = '<li>' + detail.Condition + '</li>'
        // var condInfo = '<li><img src="' + detail.Icon + '" alt="batman sucks"/></li></ul>'
        while (Math.abs(e.lngLat.lng - coordinates[0]) > 180) {
            coordinates[0] += e.lngLat.lng > coordinates[0] ? 360 : -360
        }

        new mapboxgl.Popup({anchor: 'bottom', offset: [0,-25]})
            .setLngLat(coordinates)
            .setHTML(popHtml)
            .addTo(map)

        map.flyTo({center: e.features[0].geometry.coordinates, zoom: 9, speed: 0.75, curve: 1})
    })

    },
    loadWeather() {
      let payload

      // local proxy to middleware (see /config/index.js proxyTable)
      const myRequest = new Request('/api/weather/current')

      fetch(myRequest)
      .then((response) => { 
        return response.json() })
      .then((data) => {
        
        for (var i = 0; i < data.payload.length; i++) {
          vm.addLayer(data.payload[i])
        }
        
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

// .weatherouter * .mapboxgl-popup-close-button{
//   font-size: 18px;
//   position: absolute;
//   right: -3px;
//   top: -3px;
//   color: #FFF;
//   border: 0;
//   cursor: pointer;
//   background-color: transparent;
// }

// .weatherouter * .mapboxgl-popup-content{
//   background-color:rgba(80, 80, 80,0.8);
//   border-radius: 0;
//   border: none;
//   color: #FFF;
//   font-size: 14px;
//   font-family: 'Muli', "Helvetica", Arial, sans-serif;
//   padding: 18px;
//   position: relative;
//   width: 180px;
//   -webkit-box-shadow: 0 4px 4px rgba(0, 0, 0, 0.3);
//   box-shadow: 0 4px 4px rgba(0, 0, 0, 0.3);
//   padding: 8px;
//   pointer-events: auto;
// }
// .weatherouter * .mapboxgl-popup-content ul {
//   padding-left: 0px;
// }

// .weatherouter * .mapboxgl-popup-content li{
//   padding:3px;
//   list-style:none;
// }

// .weatherouter * .mapboxgl-popup-content h2{
//   font-family: 'Muli', "Helvetica", Arial, sans-serif;
//   margin-top: 4px;
//   font-size: 28px;
//   color: #FFF;
//   padding-bottom:0px !important;
// }
// .weatherouter * .mapboxgl-popup-content ul li strong {
//   color: #FFF;
//   font-size: 16px;
// }

// button.mapboxgl-popup-close-button::before { 
//     content: "√";
// }
// .weatherouter * .mapboxgl-popup-anchor-bottom .mapboxgl-popup-tip {
//   -webkit-align-self: center;
//   align-self: center;
//   border-bottom: none;
//   border-top-color:#FFF;
// }

</style>
