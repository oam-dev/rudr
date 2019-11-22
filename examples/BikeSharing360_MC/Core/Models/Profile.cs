using System;
using System.Collections.Generic;
using System.Text;

namespace Core.Models
{
    public class Profile
    {
        public Guid Id { get; set; }
        public string FirstName { get; set; }
        public string LastName { get; set; }
        public string Email { get; set; }
        public string City { get; set; }
    }
}
