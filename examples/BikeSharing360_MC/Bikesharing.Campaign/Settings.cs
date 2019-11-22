using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace Bikesharing.Campaign
{
    public class Settings
    {

        public static string PROFILES_URL { get; internal set; }
        public static string EMAIL_URL { get; internal set; }
    }
}
