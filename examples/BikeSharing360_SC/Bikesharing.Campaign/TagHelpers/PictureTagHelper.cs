using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.Razor.TagHelpers;
using Microsoft.AspNetCore.Mvc.Rendering;
using Microsoft.AspNetCore.Mvc.Routing;
using Microsoft.AspNetCore.Mvc.ViewFeatures;
using Microsoft.AspNetCore.Razor.TagHelpers;
using System.Text.Encodings.Web;

namespace Bikesharing.Campaign.TagHelpers
{
    [HtmlTargetElement("picturex")]
    public class PictureTagHelper : TagHelper
    {
        private readonly IUrlHelperFactory _urlHelperFactory;

        public PictureTagHelper(IUrlHelperFactory urlHelperFactory)
        {
            _urlHelperFactory = urlHelperFactory;
        }

        public bool Pre { get; set; }
        public string Name { get; set; }
        private string Alt { get; set; }


        [HtmlAttributeNotBound]
        [ViewContext]
        public ViewContext ViewContext { get; set; }

        public override void Process(TagHelperContext context, TagHelperOutput output)
        {
            output.TagName = "picture";
            output.Attributes.SetAttribute("class", Pre ?  "u-pre" : "" );
            output.Attributes.SetAttribute("style", Pre ? $"background-image:url('{GetBackgroundUrlPre()}')" : "");

            // output.Content.AppendHtml(GetSourceTagHtml(480, generateMedia: true));
            // output.Content.AppendHtml(GetSourceTagHtml(960, generateMedia: true));
            output.Content.AppendHtml(GetSourceTagHtml(0, generateMedia: false));
            output.Content.AppendHtml(GetImgTagHtml());
        }

        private string GetImgTagHtml()
        {
            var urlHelper = _urlHelperFactory.GetUrlHelper(ViewContext);
            var srcName = $"{Name}.png";
            var srcUrl = urlHelper.Content($"~/images/{srcName}");
            return $"<img src=\"{srcUrl}\" alt=\"{(Alt ?? Name)}\">";
        }

        private string GetBackgroundUrlPre()
        {
            var urlHelper = _urlHelperFactory.GetUrlHelper(ViewContext);
            return urlHelper.Content($"~/images/{Name}@pre.png");
        }
        private string GetSourceTagHtml(int maxWidth, bool generateMedia)
        {
            var urlHelper = _urlHelperFactory.GetUrlHelper(ViewContext);
            var img1xName = generateMedia ? $"{maxWidth}_{Name}.png" : $"{Name}.png";
            var img2xName = generateMedia ? $"{maxWidth}_{Name}@2x.png" :$"{Name}@2x.png";
            var img3xName = generateMedia ? $"{maxWidth}_{Name}@3x.png" : $"{Name}@3x.png";

            var imgUrl1x = urlHelper.Content($"~/images/{img1xName}");
            var imgUrl2x = urlHelper.Content($"~/images/{img2xName}");
            var imgUrl3x = urlHelper.Content($"~/images/{img3xName}");
            var media = $"media=\"(max-width: {maxWidth}px)\"";
            return $"<source srcset=\"{imgUrl1x} 1x, {imgUrl2x} 2x, {imgUrl3x} 3x\" {(generateMedia ? media : string.Empty)}>";
        }

    }
}
