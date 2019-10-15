<template>
  <div class="content">
    <div class="container-fluid">
        <transition name="fade" mode="out-in">

          <router-view></router-view>
        </transition>
    </div>
  </div>
</template>
<script>
export default {};
</script>
<style>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.1s;
}

.content{
  min-height: calc(100% - 10px);
}


.fade-enter,
  .fade-leave-to
 {
  opacity: 0;
}
</style>
