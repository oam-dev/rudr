import Notify from "vue-notifyjs";
import SideBar from "@/components/SidebarPlugin";
import "es6-promise/auto";

//css assets
import "bootstrap/dist/css/bootstrap.css";
import "@/assets/sass/paper-dashboard.scss";
import "@/assets/css/themify-icons.css";

export default {
  install(Vue) {
    Vue.use(SideBar);
    Vue.use(Notify);
  }
}
