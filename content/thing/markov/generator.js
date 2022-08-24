var Module =
{
    onRuntimeInitialized: function()
    {
        document.getElementById("jsonfile").addEventListener("change", function() { Module.fetchJSON(); } );

        Module.fetchJSON();
    },

    fetchJSON: function()
    {
        var url = document.getElementById("jsonfile").value;
        var xhr = new XMLHttpRequest();
        xhr.open("GET", url, true);
        xhr.onload = function (e)
        {
            if (xhr.readyState === 4)
            {
                if (xhr.status === 200)
                {
                    Module.loadJSON(xhr.responseText);
                    Module.newWord();
                    document.getElementById("regen").addEventListener("click", function() { Module.newWord(); } );
                    document.getElementById("loader").style.display = "none";
                }
                else
                {
                    console.error(xhr.statusText);
                }
            }
        };
        xhr.onerror = function (e)
        {
            console.error(xhr.statusText);
        };
        document.getElementById("loader").style.display = "block";
        xhr.send(null);
    },

    newWord: function()
    {
        var min = 5;
        var max = 12;
        var length = Math.floor(Math.random() * (max - min + 1)) + min;
        document.getElementById("word").innerHTML = Module.generate(length);
    }
};
