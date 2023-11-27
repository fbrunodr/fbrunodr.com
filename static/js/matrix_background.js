// Get the canvas node and the drawing context
const backgroundCanvas = document.getElementById('background');
const backgroundCtx = backgroundCanvas.getContext('2d');

// set the width and height of the canvas
const w = backgroundCanvas.width = 4000;
const h = backgroundCanvas.height = 3000;

// draw a black rectangle of width and height same as that of the canvas
backgroundCtx.fillStyle = '#000';
backgroundCtx.fillRect(0, 0, w, h);

const cols = Math.floor(w / 20) + 1;
const ypos = Array(cols).fill(0);

function matrix () {
	// Draw a semitransparent black rectangle on top of previous drawing
	backgroundCtx.fillStyle = '#0001';
	backgroundCtx.fillRect(0, 0, w, h);

	// Set color to green and font to 15pt monospace in the drawing context
	backgroundCtx.fillStyle = '#0f05';
	backgroundCtx.font = '15pt monospace';

	// for each column put a random character at the end
	ypos.forEach((y, ind) => {
		// generate a random character
		const text = String.fromCharCode(Math.random() * (125-47) + 47);

		// x coordinate of the column, y coordinate is already given
		const x = ind * 20;
		// render the character at (x, y)
		backgroundCtx.fillText(text, x, y);

		// randomly reset the end of the column if it's at least 100px high
		if (y > 100 + Math.random() * 10000) ypos[ind] = 0;
		// otherwise just move the y coordinate for the column 20px down,
		else ypos[ind] = y + 20;
	});
}

// render the animation at 20 FPS.
setInterval(matrix, 50);

