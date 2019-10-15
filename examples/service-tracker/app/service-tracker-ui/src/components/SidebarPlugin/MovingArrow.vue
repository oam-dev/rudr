<template>
  <div class="moving-arrow" :style="arrowStyle">
  </div>
</template>
<script>
export default {
  props: {
    moveY: {
      type: Number,
      default: 0
    }
  },
  computed: {
    /**
     * Styles to animate the arrow
     * @returns {{transform: string}}
     */
    arrowStyle() {
      return {
        transform: `translate3d(0px, ${this.moveY}px, 0px)`
      };
    }
  }
};
</script>
<style lang="scss">
$dash-bg-khaki: #EDE6E3 !default;

.moving-arrow {
  border-right: 17px solid $dash-bg-khaki;
  border-top: 17px solid transparent;
  border-bottom: 17px solid transparent;
  display: inline-block;
  position: absolute;
  left: 243px;
  top: 95px;
  transition: all 0.5s cubic-bezier(0.29, 1.42, 0.79, 1);
}
@media (max-width: 992px) {
  .moving-arrow {
    display: none;
  }
}
</style>
