<template>

    <div class="row">
      <div class="col-xl-4 col-lg-5 col-md-6">
        <user-card>

        </user-card>
        <members-card>

        </members-card>
      </div>
      <div class="col-xl-8 col-lg-7 col-md-6">
        <edit-profile-form>

        </edit-profile-form>
        <div id="errorMessage" class="text-danger"></div>
        <pre class="well" id="graphResponse"></pre>
        <pre class="well" id="accessToken"></pre>
        <pre class="well" id="accessToken"></pre>
        <button v-on:click="loginToGraph">
            Login
          </button>
      </div>

    </div>
</template>
<script>
import EditProfileForm from "./UserProfile/EditProfileForm.vue";
import UserCard from "./UserProfile/UserCard.vue";
import MembersCard from "./UserProfile/MembersCard.vue";


export default {
  data(){
    return {
    graphClientId : "38b7e0a1-1147-4e06-8e84-b0d79fe6546a",
    graphClientRedirectUri:location.origin,
    userAgentApplication: new Msal.UserAgentApplication(this.graphClientId, null, this.loginCallback, {
    redirectUri: this.graphClientRedirectUri
    }),
    graphApiEndpoint:"https://graph.microsoft.com/v1.0/me",
    graphAPIScopes:["https://graph.microsoft.com/user.read"]

    }
  },
  created() {
    let msalScript = document.createElement('script')
    msalScript.setAttribute('src','https://secure.aadcdn.microsoftonline-p.com/lib/0.1.3/js/msal.min.js')
    console.log(this.userAgentApplication);
    this.userAgentApplication.redirectUri = this.graphClientRedirectUri;
    // this.userAgentApplication = new Msal.UserAgentApplication(this.graphClientId, null, this.loginCallback, {
    // redirectUri: this.graphClientRedirectUri
    // })
  },
  components: {
    EditProfileForm,
    UserCard,
    MembersCard
  },
  methods:{
    loginToGraph: () => {
      console.log('clicked login');
       this.userAgentApplication.loginRedirect(this.graphAPIScopes);
    },
    loginCallback: (errorDesc, token, error, tokenType) => {
      if (errorDesc) {
        showError(msal.authority, error, errorDesc);
      } else {
        callGraphApi();
      }
    }
  }
};




</script>
<style>
</style>
