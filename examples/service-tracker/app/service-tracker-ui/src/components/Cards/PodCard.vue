<template>
  <card class="pod-card">
    <div class="pod-card-body">
      <div class="row" v-if="$slots.content">
        <div class="col-10">
          <slot name="content"></slot>
        </div>
      </div>
      <div class="row" v-if="$slots.footer">
        <div class="col-1">
          &nbsp;
        </div>
        <div class="col-10">
          <slot name="footer"></slot>
        </div>
        <div class="col-1">
          &nbsp;
        </div>
      </div>
      <div class="row">
        <div class="col-1">
          &nbsp;
        </div>
        <div class="col-10">
          <slot name="raw-content"></slot>
        </div>
        <div class="col-1">
          &nbsp;
        </div>
      </div>
      <div class="row">
        <div class="col-12">
          <hr class="pod-hr">
        </div>
      </div>
      <div class="row">
        <div class="col-1" v-if="$slots.image">
          <slot name="image"></slot>
        </div>
        <div class="col-8" v-if="$slots.header">
          <slot name="header"></slot>
        </div>
      </div>
    </div>
  </card>
</template>
<script>
import Card from "./Card.vue";

export default {
  name: "pod-card",
  components: {
    Card
  }
};
</script>
<style>
</style>
