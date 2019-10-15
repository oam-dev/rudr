<template>
  <section>
    <div class="row">
      <div class="col-md-6 col-xl-3">
        <h6>EARTHQUAKE MAP</h6>
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
import moment from 'moment'
//import { ObjectCard } from "@/components/index"

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
      center: [-130.915, 9.15],
      zoom: 2
    })

    map.on('load', function() {
      vm.loadQuakes()
    })

    //** show current map center lat lng on page **//
    map.on('moveend', function(e) {
      vm.latitude = map.getCenter().lat
      vm.longitude = map.getCenter().lng
      // console.log (map.getCenter())
    })

  },
  methods: {
    filterItems(type, data, cb){
      var output = {}
      output['type'] = type
      switch(type) {
          case 'yellow':
              output['data'] = data.filter(quake => quake.properties.mag < 5)
              output['icon'] = 'eq-yellow'
              output['size'] = .35
              cb(output)
              break
          case 'orange':
              output['data'] = data.filter(quake => (quake.properties.mag >= 5 && quake.properties.mag <5.4))
              output['icon'] = 'eq-orange'
              output['size'] = .4
              cb(output)
              break
          case 'red':
              output['data'] = data.filter(quake => quake.properties.mag >= 5.4)
              output['icon'] = 'eq-red'
              output['size'] = .5
              cb(output)
              break
          default:
              break
      }

    },
    addLayer(obj){
      map.addLayer(
          {
            'id': obj.type,
            'type': 'symbol',
            'source': {
              'type': 'geojson',
              'data': {
                  'type': 'FeatureCollection',
                  'features': obj.data
              }
            },
            'layout': {
              'icon-image': obj.icon,
              'icon-size': obj.size,
              'icon-allow-overlap': true,
              'text-allow-overlap':true,
            },
            'paint':{}
            })

      map.on('click', obj.type, function (e) {
        
        var pinDetail = e.features[0].properties
        var popHtml = document.getElementById('popUpHtml').innerHTML
        var popInfo = document.getElementById('popUpDetailHtml').innerHTML
        popHtml = popHtml.replace('##title##', pinDetail.mag).replace('##subtitle##', 'MAGNITUDE')
        var locale = ['No location','note']
        if ( pinDetail.place.includes(' of ') ) locale = pinDetail.place.split(' of ')
        var detailHtml = popInfo.replace('##name##',(locale[0] + ' OF').toUpperCase()).replace('##value##', locale[1].toUpperCase())
        detailHtml = detailHtml.concat(popInfo.replace('##name##','DATE/TIME').replace('##value##', moment(pinDetail.time).format('MM/DD/YYYY HH:mm a')))
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
    loadQuakes() {
      let payload
      const myRequest = new Request('/api/quakes/current')

      fetch(myRequest)
      .then((response) => { 
        return response.json() })
      .then((data) => {
        payload = data.payload
        vm.filterItems('red', payload, function(red){
          vm.addLayer(red)
        })
        vm.filterItems('orange', payload, function(orange){
          vm.addLayer(orange)
        })
        vm.filterItems('yellow', payload, function(yellow){
          vm.addLayer(yellow)
        })
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

</style>
