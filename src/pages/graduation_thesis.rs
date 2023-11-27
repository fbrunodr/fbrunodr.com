use actix_web::{get, HttpResponse, Result};

use crate::components::navbar::navbar;

#[get("/graduation_thesis")]
pub async fn render() -> Result<HttpResponse> {
    let html_content = format!("
        <html lang=\"en\">
            <head>
                <meta charset=\"utf-8\" />
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/index.css\">
                <link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">
                <link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>
                <link href=\"https://fonts.googleapis.com/css2?family=Open+Sans:wght@300;400&family=Reenie+Beanie&family=Source+Code+Pro&display=swap\" rel=\"stylesheet\">
            <head/>
            <body>
                {}

                <canvas id=\"background\">
                </canvas>

                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/graduation_thesis.css\">

                <div class=\"content\">
                    <div class=\"scroll\">
                        <h2> Hello dear reader, </h2>
                        <p>
                            It is late November 2023. A thesis, titled \"Efficient Algorithms for Solving
                            Optimization Problems in Aerospace Trajectories\" has been completed.
                            This scholarly work offers a comparison of algorithms traditionally employed
                            to navigate the complexities of space travel, scrutinized through the lens
                            of three distinct test scenarios. The document awaits your perusal here:
                        </p>

                        <a class=\"resource\" href=\"/static/tc128_2023.pdf\">
                            tc128_2023.pdf
                        </a>

                        <p>
                            For those who seek to delve deeper into the matrix of this research, the
                            source code utilized for data generation within the thesis is accessible in the
                            vast digital expanse of GitHub:
                        </p>

                        <a class=\"resource\" href=\"https://github.com/fbrunodr/TestOptimizersOnSpaceTrajectoryProblems\">
                            Graduation thesis source code
                        </a>

                        <p>
                            To run the optimizations, one must compile the ESA source code.
                            Enter the GTOPtoolbox directory (GTOP stands for Global Trajectory
                            Optimization Problems), create a directory named build, go inside it, run 
                            <span class=\"code\">cmake ..</span> then run
                            <span class=\"code\">make all</span> to compile the test problems. Further
                            instructions are available on the README.md file at the root of the repository.
                        </p>

                        <p>
                            The essence of the thesis can be distilled into a succinct conclusion:
                        </p>

                        <h1>
                            Genetic algorithms stand superior in their class. Simulated Annealing performs admirably,
                            shall the user not mind it occasionally falter in the traps of local minima. The remaining
                            algorithms showed a disappointing performance in solving the optimization
                            problems presented.
                        </h1>

                        <p>
                            Beyond the academic conclusions, the thesis reveals captivating visual outcomes of the
                            optimizations. For instance, consider the trajectory from Earth to Jupiter that
                            strategically incorporates a fly-by through Mars for fuel savings:
                        </p>

                        <img src=\"/static/earth_mars_jupiter.png\">

                        <p>
                            This maneuver requires only 9.42 km/s of delta-v, significantly less than the direct
                            route to Jupiter, which demands at least 12.57 km/s:
                        </p>

                        <img src=\"/static/earth_jupiter.png\">

                        <p>
                            The document also delves into the Cassini 1 problem, mirroring the trajectory of the historical
                            <a href=\"https://science.nasa.gov/mission/cassini/\">Cassini mission</a>.
                            The optimized route presented uses only 4.97 km/s of delta-v to reach Saturn, deftly
                            illustrating the remarkable efficiency of gravitational assists:
                        </p>

                        <img src=\"/static/cassini_1.png\">

                        <p>
                            Zooming in on the first three maneuvers of the Cassini 1 path, we observe a calculated
                            approach with two fly-bys near Venus. These maneuvers leverage the gravitational pull of
                            Venus to alter the spacecraft's trajectory efficiently:
                        </p>

                        <img src=\"/static/cassini_1_first_three_trips.png\">

                        <p>
                            These findings underscore the effectiveness of using planetary fly-bys for fuel
                            conservation in interplanetary travel.
                        </p> 

                        <p>
                            Yes, this text was generated using chatgpt, ur welcome...
                        </p>
                    </div>
                </div>
            </body>

            <script type=\"text/javascript\" src=\"/static/js/graduation_thesis.js\"></script>
        </html>
    ", navbar());

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content)) 
}
