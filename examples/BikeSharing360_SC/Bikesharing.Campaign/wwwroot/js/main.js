var Bikes;
(function (Bikes) {
    'use strict';

    // Animations1
    var WayNamespace = 'Waypoint';
    var elements = document.querySelectorAll('.wayp');
    Array.prototype.forEach.call(elements, function (element) {
        var waypoint = new window[WayNamespace]({
            element: element,
            handler: function () {
                this.element.classList.add('animated');
            },
            offset: parseInt(element.dataset.offset, 10) || 500
        });
    });

    var functionEndpoint = 'https://bikesharing-functions.azurewebsites.net/api/MarketingRequestHandler?code=o5evolq13noi3e36k5ut0529zg2ghee4yv5o5iog7qmvaemistxnody16wemc1rz5x00y66r';
    var sendEmail = function () {
        var data = {
            email: $('#email').val(),
            city: $('#city').val()
        };
        $.ajax({
            url: functionEndpoint,
            type: 'POST',
            crossDomain: true,
            data: JSON.stringify(data),
            dataType: 'json',
            contentType: 'application/json',
            error: function (xhr, status) {
                console.log('error');
            }
        });
    };

    // Preload images
    var preimages = document.querySelectorAll('.u-pre img');
    $(document).ready(function () {
        Array.prototype.forEach.call(preimages, function (image) {
            image.style.opacity = '1';
        });
        $('#submit').click(function () {
            sendEmail();
        });
    });
})(Bikes || (Bikes = {}));
