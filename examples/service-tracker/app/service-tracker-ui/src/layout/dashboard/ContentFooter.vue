<template>
  <footer class="footer">
    <div class="container-fluid d-flex flex-wrap justify-content-between">
      <nav>
        <ul>
          <li>
            <router-link :to="{path:'/admin'}">Back to Dashboard</router-link>
          </li>
        </ul>
      </nav>
      <div class="copyright d-flex flex-wrap">
        <a href="https://github.com/azure/kubernetes-hackfest" target="_blank">Github Repo&nbsp;<i class="fab fa-github"></i></a>
      </div>
    </div>
  </footer>
</template>

<script>
  export default {};
</script>

<style>
</style>
