#region using
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using System.Collections;
using System.Runtime.InteropServices;
using System.Text;
using System.Net.Http;
using Core;
using System.Diagnostics;
#endregion

namespace Bikesharing.Campaign.Controllers
{
    public class HomeController : Controller
    {
        public IActionResult Index()
        {
            return View();
        }

        [HttpPost("Home/Email")]
        public async Task<IActionResult> Email(string email, string city)
        {
                    // payload for finding/creating a profile
            var client = new HttpClient();
            var reqJson = Newtonsoft.Json.JsonConvert.SerializeObject(new
            {
                Email = email,
                City = city
            });
            var content = new StringContent(reqJson, Encoding.UTF8, "application/json");

            // Create a Profile 
            var response = await client.PostAsync(Settings.PROFILES_URL + "/api/profiles", content);
            var json = await response.Content.ReadAsStringAsync();
            var profile = Newtonsoft.Json.JsonConvert.DeserializeObject<Core.Models.Profile>(json);

            //// payload for submitting an email request
            var emailReqJson = Newtonsoft.Json.JsonConvert.SerializeObject(new
            {
               Email = profile.Email,
               FirstName = profile.FirstName
            });

            var emailContent = new StringContent(emailReqJson, Encoding.UTF8, "application/json");

            // Send Email
            var emailResponse = await client.PostAsync(Settings.EMAIL_URL + "/api/email", emailContent);
            var emailJson = await emailResponse.Content.ReadAsStringAsync();
            dynamic emailData = Newtonsoft.Json.Linq.JObject.Parse(emailJson);



            // Confirm customer is signed up
            // ViewBag.Confirmation = (string)profile.FirstName;

            ViewData["Message"] = profile.FirstName;

            return View("Index");
        }

        public IActionResult About()
        {
            // Debugging Information 
            if (Environment.GetEnvironmentVariable("ASPNETCORE_ENVIRONMENT") == "Development")
            {
                ViewData["Message"] = "Debugging Info.";

                ViewData["HOSTNAME"] = Environment.GetEnvironmentVariable("COMPUTERNAME") ??
                                                Environment.GetEnvironmentVariable("HOSTNAME");
                ViewData["OSARCHITECTURE"] = RuntimeInformation.OSArchitecture;
                ViewData["OSDESCRIPTION"] = RuntimeInformation.OSDescription;
                ViewData["PROCESSARCHITECTURE"] = RuntimeInformation.ProcessArchitecture;
                ViewData["FRAMEWORKDESCRIPTION"] = RuntimeInformation.FrameworkDescription;
                ViewData["ASPNETCORE_ENVIRONMENT"] = Environment.GetEnvironmentVariable("ASPNETCORE_ENVIRONMENT");

                StringBuilder envVars = new StringBuilder();
                foreach (DictionaryEntry de in Environment.GetEnvironmentVariables())
                    envVars.Append(string.Format("<strong>{0}</strong>:{1}<br \\>", de.Key, de.Value));

                ViewData["ENV_VARS"] = envVars.ToString();
            }

            return View();
        }  
        public IActionResult Contact()
        {
            ViewData["Message"] = "Your contact page.";

            return View();
        }

        public IActionResult Error()
        {
            return View();
        }
    }
}
