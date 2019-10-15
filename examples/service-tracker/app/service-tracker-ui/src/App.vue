<template>
  <div :class="{'nav-open': $sidebar.showSidebar}">
    
    <notifications></notifications>
    <router-view></router-view>

    <section modal>
      <div class="modal fade" id="actionModal" tabindex="-1" role="dialog" aria-hidden="true" aria-labelledby="actionModal">
        <div class="modal-dialog" role="document">
          <div class="modal-content">
            <div class="modal-header">
              <h5 class="modal-title"><i class="fas fa-database text-warning pr-2"></i>Refreshing Data</h5>
              <button type="button" id="actionModalClose" class="close" data-dismiss="modal" aria-label="Close">
                  <span aria-hidden="true"><i class="fas fa-angle-double-down fa-xs"></i></span>
              </button>
            </div>
            <div class="modal-body">
              <h5>Data retrieval in progress...</h5>
              <div class="progress">
                  <div class="progress-bar progress-bar-striped progress-bar-animated bg-success" role="progressbar" aria-valuenow="100" aria-valuemin="0" aria-valuemax="100" style="width: 100%"></div>
              </div>
            </div>
            <div class="modal-footer">
              <h6><i class="fas fa-book-reader fa-xs text-info pr-2"></i>While you wait</h6>
              <p class="did-you-know">Did you know the human head weighs 8 pounds?</p>
            </div>
          </div>
        </div>
      </div>
    </section>
    
    <section card style="display:none" id="popUpHtml">
      <div class="card border-primary">
          <div class="card-body">
              <footer class="blockquote-footer popTitleHead">##subtitle##</footer>
              <h5 class="card-title">##title##</h5><i class="fas fa-angle-double-down fa-xs closePop"></i>
          </div>
          <ul class="list-group">
            ##info##
          </ul>
      </div>
    </section>

    <section cardInfo style="display:none" id="popUpDetailHtml">
      <li class="list-group-item pl-3"><footer class="blockquote-footer">##name##</footer><p class="mb-0 popValue">##value##</p></li>
    </section>

    <section cardInfoImg style="display:none" id="popUpDetailImage">
      <li class="list-group-item text-center mb-3">##image##</li>
    </section>

  </div>
</template>

<script>
export default {};
</script>

<style lang="scss">
@import url("https://api.tiles.mapbox.com/mapbox-gl-js/v0.48.0/mapbox-gl.css");

.modal-dialog {
  position: relative;
  top: 120px;
}

/*      light colors - used for select dropdown         */
.list-move {
  transition: transform 0.3s, opacity 0.4s;
}

.list-item {
  display: inline-block;
  margin-right: 10px;
}

.list-enter-active,
.list-leave-active {
  transition: opacity 0.4s;
}

.list-enter,
.list-leave-to {
  opacity: 0;
}

.vue-notifyjs button.close {
  -webkit-appearance: none;
  padding: 0;
  cursor: pointer;
  background: transparent;
  border: 0;
}

.vue-notifyjs .close {
  float: right;
  font-size: 21px;
  font-weight: bold;
  line-height: 1;
  color: #000;
  text-shadow: 0 1px 0 #fff;
  filter: alpha(opacity=20);
  opacity: 0.2;
}

.vue-notifyjs .alert {
  border: 0;
  border-radius: 0;
  color: #ffffff;
  padding: 10px 15px;
  font-size: 14px;
  z-index: 100;
  display: inline-block;
  position: fixed !important;
  transition: all 0.5s ease-in-out;
  cursor: pointer;
}
.vue-notifyjs .alert.center {
  left: 0px;
  right: 0px;
  margin: 0 auto;
}
.vue-notifyjs .alert.left {
  left: 20px;
}
.vue-notifyjs .alert.right {
  right: 20px;
}
.container .vue-notifyjs .alert {
  border-radius: 4px;
}
.navbar .vue-notifyjs .alert {
  border-radius: 0;
  left: 0;
  position: absolute;
  right: 0;
  top: 0px !important;
  width: 100%;
  z-index: 3;
}
.navbar:not(.navbar-transparent) .vue-notifyjs .alert {
  top: 70px;
}
.vue-notifyjs .alert .alert-icon {
  font-size: 30px;
  margin-right: 5px;
}
.vue-notifyjs .alert .close ~ span {
  display: block;
  max-width: 89%;
}
.vue-notifyjs .alert[data-notify="container"] {
  width: 350px;
  padding: 10px 10px 10px 20px;
  border-radius: 4px;
}
.vue-notifyjs .alert.alert-with-icon {
  padding-left: 65px;
}

.vue-notifyjs .alert-info {
  background-color: #7ce4fe;
  color: #3091b2;
}

.vue-notifyjs .alert-success {
  background-color: #6bbd00 !important;
  border: 1px solid #6ab30c !important;
  border-radius: 0px !important;
  font-family: "Bangers", cursive !important;
  font-size: 18px !important;
  color: #fff !important;
  top: 0px !important;
-webkit-box-shadow: 1px 0px 4px 2px rgba(153, 153, 153, 0.8);
-moz-box-shadow: 1px 0px 4px 2px rgba(153, 153, 153, 0.8);
box-shadow: 1px 0px 4px 2px rgba(153, 153, 153, 0.8);
}
.vue-notifyjs .alert-success button {
  font-size: 24px;
  color: #fff !important;
  font-family: "Trebuchet MS", "Lucida Sans Unicode", "Lucida Grande",
    "Lucida Sans", Arial, sans-serif !important;
}

.vue-notifyjs .alert-warning {
  background-color: #ffe28c;
  color: #bb992f;
}

.vue-notifyjs .alert-danger {
  background-color: #ff8f5e;
  color: #b33c12;
}
</style>
