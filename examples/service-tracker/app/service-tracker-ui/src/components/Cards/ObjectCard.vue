<template>
    <section objectCard class="obj-card">
        <div class="obj-card-body">
            <div class="row obj-row-header">
                <div class="col-12 obj-name">
                    <h5>{{objName}}</h5>
                    <small class="obj-type"><i :class="objSubTitleIcon"></i>&nbsp;&nbsp;{{objSubTitle}}</small>
                </div>
            </div>

            <div class="row obj-row-content" v-if="objInfoArray">
                <ul class="list-group list-group-flush">
                    <li class="list-group-item" v-for="info in objInfoArray" :key="info.name">
                        <footer class="blockquote-footer popInfoHead pb-1">{{info.name}}</footer>
                        <span class="pt-1">{{info.value}}</span></li>
                </ul>
            </div>

            <div class="row obj-row-data-status">
                <ul class="list-group list-group-flush">
                    <li class="list-group-item" v-if="objServiceAvailable">
                        <footer class="blockquote-footer popInfoHead pb-1">LATEST DATA</footer>
                        <span class="pt-1">{{objLatestTimeStamp}}</span></li>
                    <li class="list-group-item" v-else>
                        <footer class="blockquote-footer popInfoHead pb-1">NO DATA PRESENT</footer>
                        <span class="pt-1">Click Refresh</span></li>
                    <li class="list-group-item">
                        <button type="button" :id="objName" class="btn btn-outline-success btn-sm refreshDataModal" v-on:click="showModal(objName)">
                            <i class="fas fa-sync-alt text-success pr-1"></i>Refresh Data
                        </button>
                    </li>
                </ul>
            </div>

            <div class="row obj-row-footer">
                <div class="col-8">
                    <i :class="objStatusIcon"></i><span class="obj-status">
                        {{objStatus}}
                    </span>
                </div>
                <div class="col-4">&nbsp;</div>
            </div>
        </div>

    </section>
</template>

<script>
import modal from 'bootstrap/js/dist/modal'
import $ from 'jquery'
let vm


export default {
  name: "object-card",
  props: {
    objLatestTimeStamp: String,
    objName: String, // NAME OF THE OBJECT
    objSubTitle: String, // TYPE or SUBTITLE OF THE OBJECT (POD, NODE, ETC)
    objSubTitleIcon: String, // SMALL ICON (fa fa-xxxx) OF THE OBJECT
    objInfoArray: Array, // DETAIL NAME OF THE OBJECT (IP, HOSTIP, ETC)
    objStatusIcon: String, // ICON OF THE STATUS
    objStatusIconClass: String, // COLOR CLASS OF THE ICON OF THE STATUS
    objStatus: String, // ACTUAL STATUS NAME
    statusColor: "#28a745",
    objServiceAvailable: Boolean
  },
  created(){
      vm = this
  },
  data(){
      return {
    dyk:['Did you know the Intel i7-6950X has 10 cores?', 
      'Did you know that .rs is the top-level domain for Serbia?', 
      'Did you know that Whistler was the codename for the Microsoft Windows XP OS?',
      'Did you know that Java was developed by Sun Microsystems in 1995?'],
      facts : {}
      }
      
  },
  methods:{
    showModal(name){
        $('#actionModal').modal('show')
        vm.facts = setInterval(dykTimer, 6000)
        this.$parent.refresh(name)
        function dykTimer() {
          var itemNum = Math.floor(Math.random() * Math.floor(vm.dyk.length))
          $('.did-you-know').text(vm.dyk[itemNum])
        }        
    },
    hideModal(){
        $('#actionModal').modal('hide')
        clearInterval(vm.facts)
    }
  }
};
</script>
<style>
</style>


