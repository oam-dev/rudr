using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using Core.Models;

// For more information on enabling Web API for empty projects, visit https://go.microsoft.com/fwlink/?LinkID=397860

namespace Profile.Api.Controllers
{
    [Route("api/profiles")]
    public class ProfileController : Controller
    {

        // POST api/profile
        [HttpPost]
        public IActionResult Post([FromBody]dynamic value)
        {
            Core.Models.Profile profile = FindCreateProfile(
                (string)value.Email, 
                (string)value.City);
            return Ok(profile);
        }














       Core.Models.Profile FindCreateProfile(string email, string city)
        {
            return new Core.Models.Profile
            {
                Id = Guid.NewGuid(),
                FirstName = "Scott",
                LastName = "Hanselman",
                Email = email,
                City = city
            };
        }
    }
}
