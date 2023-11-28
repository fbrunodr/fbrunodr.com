function handleResize() {
    const scrollable = document.querySelector('.scroll');

    const p = 300;
    const h = window.innerHeight;
    const alpha = 17 * Math.PI / 180;
    const theta = 2 * Math.atan2(h / 2, p);

    const k = Math.cos(theta / 2 + alpha) / Math.cos(theta / 2);
    const h_prime = h * k;
    const p_prime = p * k;

    const x = p_prime - p + h/2 * Math.sin(alpha);
    const y = h / 2 * Math.cos(alpha) - h_prime / 2;

    const screenWidth = window.innerWidth;

    if(screenWidth >= 1024)
        scrollable.style.width = `${50*k}%`;
    else if(screenWidth >= 768)
        scrollable.style.width = `${75*k}%`;
    else
        scrollable.style.width = `${100*k}%`;

    scrollable.style.transform = `perspective(${p}px) translateY(${-y}px) translateZ(${-x}px) rotateX(${alpha}rad)`;
}

window.addEventListener('resize', handleResize)

// Initially call the function to handle the initial dimensions
handleResize();

// Function to continuously scroll the content
function scrollContent() {
    const scrollable = document.querySelector('.scroll');
    scrollable.scrollTop += 1;
}

setInterval(scrollContent, 1000 / 30);
