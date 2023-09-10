pub fn navbar() -> String {
    let navbar = String::from("
        <script type=\"text/javascript\" src=\"/static/js/navbar.js\"></script>
        <script src=\"https://kit.fontawesome.com/7178fd9426.js\" crossorigin=\"anonymous\"></script>
        <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/navbar.css\">
        <div id=\"navbarDiv\">
        </div>
        <script type=\"text/javascript\">
            navbar_handler()
        </script>
    ");

    navbar
}
