<template>
  <div>
    <div class="row">
          <div class="col-md-6 col-xl-3">
            <h6>SERVICE STATUS</h6>
          </div>
    </div>

    <!--Service Status-->
    <div class="row">
      <!-- Add various services here -->
      <div class="col-md-3 col-xl-3" v-for="service in serviceCards" :key="service.objName">
        <object-card :ref="service.objName"
          :obj-name="service.objName"
          :obj-sub-title-icon="service.objSubTitleIcon"
          :obj-sub-title="service.objSubTitle"
          :obj-info-array="service.objInfoArray"
          :obj-status-icon="service.objStatusIcon"
          :obj-status="service.objStatus"
          :obj-service-available="service.objServiceAvailable"
          :obj-latest-time-stamp="service.objLatestTimeStamp"
        />
      </div>

    </div>

    <!--Pod cards-->
    <div class="row">
      <div class="col-md-6 col-xl-3" v-for="pod in podCards" :key="pod.objName">
        <object-card 
          :obj-name="pod.objName"
          :obj-type="pod.objType"
          :obj-info-array="pod.objInfoArray"
          :obj-status-icon="pod.objStatusIcon"
          :obj-status="pod.objStatus"/>
      </div>
    </div>



    <!-- Stats cards
    <div class="row">
      <div class="col-md-6 col-xl-3" v-for="stats in statsCards" :key="stats.title">
        <stats-card>
          <div class="icon-big text-center" :class="`icon-${stats.type}`" slot="header">
            <i :class="stats.icon"></i>
          </div>
          <div class="numbers" slot="content">
            <p>{{stats.title}}</p>
            {{stats.value}}
          </div>
          <div class="stats" slot="footer">
            <i :class="stats.footerIcon"></i> {{stats.footerText}}
          </div>
        </stats-card>
      </div>


    </div>

    Charts
    <div class="row">

      <div class="col-12">
        <chart-card title="Users behavior"
                    sub-title="24 Hours performance"
                    :chart-data="usersChart.data"
                    :chart-options="usersChart.options">
          <span slot="footer">
            <i class="ti-reload"></i> Updated 3 minutes ago
          </span>
          <div slot="legend">
            <i class="fa fa-circle text-info"></i> Open
            <i class="fa fa-circle text-danger"></i> Click
            <i class="fa fa-circle text-warning"></i> Click Second Time
          </div>
        </chart-card>
      </div>

      <div class="col-md-6 col-12">
        <chart-card title="Email Statistics"
                    sub-title="Last campaign performance"
                    :chart-data="preferencesChart.data"
                    chart-type="Pie">
          <span slot="footer">
            <i class="ti-timer"></i> Campaign set 2 days ago</span>
          <div slot="legend">
            <i class="fa fa-circle text-info"></i> Open
            <i class="fa fa-circle text-danger"></i> Bounce
            <i class="fa fa-circle text-warning"></i> Unsubscribe
          </div>
        </chart-card>
      </div>

      <div class="col-md-6 col-12">
        <chart-card title="2015 Sales"
                    sub-title="All products including Taxes"
                    :chart-data="activityChart.data"
                    :chart-options="activityChart.options">
          <span slot="footer">
            <i class="ti-check"></i> Data information certified
          </span>
          <div slot="legend">
            <i class="fa fa-circle text-info"></i> Tesla Model S
            <i class="fa fa-circle text-warning"></i> BMW 5 Series
          </div>
        </chart-card>
      </div>

    </div> -->

  </div>
</template>
<script>
import { ObjectCard } from "@/components/index"

String.prototype.toProperCase = function () {
    return this.replace(/\w\S*/g, function(txt){return txt.charAt(0).toUpperCase() + txt.substr(1).toLowerCase()})
}

let vm

export default {
  components: {
    ObjectCard
  },

  data() {
    return {
      podCards:[],
      currentServices:['flights', 'quakes', 'weather'],
      serviceCards:[]
    }
  },
  created() {
    vm = this

    this.currentServices.forEach((svc) => {
      // var svcname = svc.toUpperCase()

      var req = new Request('/api/' + svc + '/status')
      fetch(req)
      .then((res) => {
        return res.json()
      })
      .then((data) => {
        var info = []
        info.push({name:'Uptime', value: data.payload.uptime})
        this.serviceCards.push({objServiceAvailable:true,objLatestTimeStamp:data.payload.latest, objName:svc.toProperCase(),objSubTitle:'Microservice', objInfoArray: info, objSubTitleIcon:'fas fa-project-diagram text-muted', objStatusIcon: 'fas fa-check-circle text-success', objStatus:'Available' })
      })
      .catch((e) =>{
        console.log(e)
        this.serviceCards.push({objServiceAvailable:false, objName:svc.toProperCase(),objSubTitle:'Microservice', objSubTitleIcon:'fas fa-project-diagram text-muted', objStatusIcon: 'fas fa-check-circle text-danger', objStatus:'Error' })
      })

    })


    //const myRequest = new Request('/api/pods')
    // fetch(myRequest)
    // .then((response) => { 
    //   return response.json()
    // })
    // .then((data) => {
    //   console.log(data)
    //   this.statsCards.length = 0
    //   var podIp = {name: "Pod IP", value: data[0].podIP + ":" + data[0].ports.substring(6)}
    //   var hostIp = {name: "Host IP", value: data[0].hostIP}
    //   var statusIcon = "fa fa-adjust fa-lg text-warning" // pending or other
    //   if (data[0].status.toUpperCase() === "RUNNING"){
    //     statusIcon = "fa fa-check-circle fa-lg text-success"
    //   }
    //   var pod = {
    //     objName: data[0].name,
    //     objType: "pod",
    //     objInfoArray: [podIp, hostIp],
    //     objStatusIcon : statusIcon,
    //     objStatus: data[0].status.toUpperCase()
    //   };
    //   this.podCards.push(pod)


    // })

    // flight stats

    // const flightOverview = new Request('/api/stats/inair')

    // fetch(flightOverview)
    // .then((response) => { 
    //   return response.json()
    // })
    // .then((data) => {
    //   var info = []
    //   for(var i=0; i<5; i++){
    //     info.push({name: data[i].country, value: data[i].total})
    //   }
    //   this.flightCards.push({
    //     objName: 'Top 5 Countries',
    //     objSubTitle: 'Flights in-air',
    //     objSubTitleIcon: 'fa fa-plane ',
    //     objInfoArray: info,
    //     objStatusIcon : 'fa fa-globe-americas fa-lg text-success',
    //     objStatus: 'Worldwide'
    //     })
    // })


  },
  mounted(){

  },
  methods:{
    showNotification(msg){
      this.$notify({
        message: msg,
        horizontalAlign: 'center',
        verticalAlign: 'top',
        type: 'success',
        closeOnClick: true,
        timeout: 5000
      })
    },
    refresh(name){
      var refReq = new Request('/api/' + name.toLowerCase() + '/refresh')
      fetch(refReq)
      .then((refRes) => {
        if(refRes.ok) {
          vm.$refs[name][0].hideModal()
          vm.showNotification(name + ' data refreshed successfully')
          return refRes.json()
        }
        throw new Error('Network response was not ok.')
      })
      .then((data) =>{
        console.log(data.payload.payload.Timestamp)
      })
      .catch((e) =>{
        alert('oh snap, something went wrong with')
        console.log(e)
      })
    }
  }
};
</script>
<style>
</style>
