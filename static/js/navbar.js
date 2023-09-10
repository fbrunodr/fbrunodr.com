navbar_state = 1;

navbar_handler = () => {
    navbar_state ^= 1;

    if(navbar_state) {
        document.getElementById("navbarDiv").innerHTML = `
            <nav class="navbar dark">
                <a class="navbar-logo" href="/">Francisco Bruno</a>

                <div class="navbar-icon" onClick="navbar_handler()">
                    <i class="fas fa-times"></i>
                </div>

                <div class="navbar-items active">
                    <a class="nav-link" href="/"> Home </a>
                    <a class="nav-link" href="/static/Resume.pdf"> Resume </a>
                </div>

                <div class="social-items active">
                    <a class="social-link" href="https://linkedin.com/in/francisco-bruno-dias-ribeiro-da-silva/" >
                        <i class="fab fa-linkedin"></i>
                    </a>
                    <a class="social-link" href="https://instagram.com/fbrunodr/" >
                        <i class="fab fa-instagram"></i>
                    </a>
                    <a class="social-link" href="mailto:fbrunodr@gmail.com">
                        <i class="far fa-envelope"></i>
                    </a>
                </div>

            </nav>
        `;
    }
    else{
        document.getElementById("navbarDiv").innerHTML = `
        <nav class="navbar">
            <a class="navbar-logo" href="/">Francisco Bruno</a>

            <div class="navbar-icon" onClick="navbar_handler()">
                <i class="fas fa-bars"></i>
            </div>

            <div class="navbar-items">
                <a class="nav-link" href="/"> Home </a>
                <a class="nav-link" href="/static/Resume.pdf"> Resume </a>
            </div>

            <div class="social-items">
                <a class="social-link" href="https://linkedin.com/in/francisco-bruno-dias-ribeiro-da-silva/" >
                    <i class="fab fa-linkedin"></i>
                </a>
                <a class="social-link" href="https://instagram.com/fbrunodr/" >
                    <i class="fab fa-instagram"></i>
                </a>
                <a class="social-link" href="mailto:fbrunodr@gmail.com">
                    <i class="far fa-envelope"></i>
                </a>
            </div>

        </nav>
    `;
    }
}