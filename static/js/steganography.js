const dropArea = document.getElementById("drop-area");
const inputFile = document.getElementById("input-file");
const imageView = document.getElementById("img-view");

let imageViewHeight = 0;
let imgLink;

inputFile.addEventListener("change", uploadImage);

function uploadImage(){
	if(imageViewHeight === 0){
		imageViewHeight = imageView.clientHeight;
		imageView.style.height = imageViewHeight;
	}
	imgLink = URL.createObjectURL(inputFile.files[0]);
	imageView.style.backgroundImage = `url(${imgLink})`;
	imageView.textContent = "";
	imageView.style.border = 0;
}


dropArea.addEventListener("dragover", function(e){
	e.preventDefault();
});


dropArea.addEventListener("drop", function(e){
	e.preventDefault();
	inputFile.files = e.dataTransfer.files;
	uploadImage();
});


function encryptMessage(message, password) {
    const ciphertext = CryptoJS.AES.encrypt(message, password);
    return ciphertext.toString();
}


function decryptMessage(ciphertext, password) {
    const bytes = CryptoJS.AES.decrypt(ciphertext, password);
    const originalText = bytes.toString(CryptoJS.enc.Utf8);
    return originalText;
}


function stringToBits(inputString) {
	const bits = [];

	for (let i = 0; i < inputString.length; i++) {
		const charCode = inputString.charCodeAt(i);

		// Convert the character code to its binary representation (8 bits)
		const binaryString = charCode.toString(2);
		const paddedBinary = binaryString.padStart(8, '0');

		for (let j = 0; j < paddedBinary.length; j++) {
			bits.push(parseInt(paddedBinary[j]));
		}
	}

	return bits;
}


function bitsToString(bitsArray) {
	if (bitsArray.length % 8 !== 0) {
		throw new Error("Input does not have a valid binary representation");
	}

	let reconstructedString = "";

	for (let i = 0; i < bitsArray.length; i += 8) {
		const byteBits = bitsArray.slice(i, i + 8);
		const decimalValue = parseInt(byteBits.join(""), 2);
		const char = String.fromCharCode(decimalValue);
		reconstructedString += char;
	}

	return reconstructedString;
}


function embedMessageInImage() {
	const message = document.getElementById("message-field").value;
	const password = document.getElementById("password-field").value;

	if(!imgLink){
		alert("Upload image to proceed");
		return;
	}

	if(!message){
		return;
	}

	img = new Image();
	img.src = imgLink;
	img.onload = () => {
		const canvas = document.createElement('canvas');
		canvas.width = img.width;
		canvas.height = img.height;
		const ctx = canvas.getContext('2d');
		ctx.drawImage(img, 0, 0);
		const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
		let data = imageData.data;

		let ciphertext = message;
		if(password){
			ciphertext = encryptMessage(message, password);
		}
		const ciphertextBits = stringToBits(ciphertext + '\0');

		let i = 0, j = 0;

		while(j < ciphertextBits.length){
			if(i >= data.length){
				alert("Image does not have enough opaque pixels");
				canvas.remove();
				return;
			}

			if(i % 4 == 0 && data[i + 3] != 255)
				i += 4;
			else if(i % 4 == 3)
				i++;
			else{
				data[i] = (data[i] & ~1) | ciphertextBits[j];
				i++;
				j++;
			}
		}

		ctx.putImageData(imageData, 0, 0);
		const downloadLink = document.createElement('a');
		downloadLink.href = canvas.toDataURL();
		downloadLink.download = 'hiddenMessage.png';
		document.body.appendChild(downloadLink);
		downloadLink.click();
		document.body.removeChild(downloadLink);
		canvas.remove();
	}
}


function extractMessageFromImage() {
	const password = document.getElementById("password-field").value;

	if(!imgLink){
		alert("Upload image to proceed");
		return;
	}

	img = new Image();
	img.src = imgLink;
	img.onload = () => {
		let canvas = document.createElement('canvas');
		canvas.width = img.width;
		canvas.height = img.height;
		let ctx = canvas.getContext('2d');
		ctx.drawImage(img, 0, 0);
		let imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
		let data = imageData.data;

		const bits = [];
		let i = 0;
		let zerosInRow = 0;

		while(!(bits.length % 8 == 0 && zerosInRow >= 8)){
			if(i >= data.length){
				break;
			}
			if(i % 4 == 0 && data[i+3] != 255)
				i += 4;
			else if(i % 4 == 3)
				i++;
			else{
				const bit = data[i] & 1;
				bits.push(bit);
				if(bit)
					zerosInRow = 0;
				else
					zerosInRow++;
				i++;
			}
		}

		canvas.remove();
		const encryptedMessage = bitsToString(bits).slice(0,-1);
		let message = encryptedMessage;
		if(password){
			message = decryptMessage(encryptedMessage, password);
		}

		const responseDiv = document.getElementById("response");
		responseDiv.style.backgroundColor = "rgba(0, 0, 0, 0.5)";
		responseDiv.innerText = `Hidden Text:\n${message}`;
	}
}

function set_encrypt_mode(){
	document.getElementById("encrypt-decrypt").innerHTML = `
		<form id=\"data-form\" onsubmit="event.preventDefault(); return embedMessageInImage()">
			<div class=\"item\">
				<label for="message-field">Message*:</label>
				<input type="text" id="message-field" name="message-field" required>
			</div>
			<div class=\"item\">
				<label for="password-field">Password:</label>
				<input type="password" id="password-field" name="password-field">
				<br>
			</div>
			<input class=\"button\" type="submit" value="Encrypt">
		</form>
	`;
	document.getElementById("encrypt-switch").style = "color: black; background-color: #c71c63";
	document.getElementById("decrypt-switch").style = "color: #c71c63; background-color: #36384c";
}

function set_decrypt_mode(){
	document.getElementById("encrypt-decrypt").innerHTML = `
		<form id=\"data-form\" onsubmit="event.preventDefault(); return extractMessageFromImage()">
			<div class=\"item\">
				<label for="password-field">Password:</label>
				<input type="password" id="password-field" name="password-field">
			</div>
			<input class=\"button\" type="submit" value="Decrypt">
		</form>
	`;
	document.getElementById("encrypt-switch").style = "color: #c71c63; background-color: #36384c";
	document.getElementById("decrypt-switch").style = "color: black; background-color: #c71c63";
}
