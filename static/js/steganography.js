const dropArea = document.getElementById("drop-area");
const inputFile = document.getElementById("input-file");
const imageView = document.getElementById("img-view");
const button = document.getElementById("encrypt-decrypt-button");

let state = 0;
let imageViewHeight = 0;
let imgLink;

inputFile.addEventListener("change", uploadImage);
button.addEventListener("click", function(){
	if(state){
		extractMessageFromImage();
	}
	else{
		embedMessageInImage();
	}
});


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
	const message = 'Hello World';
	const password = '123';

	if(!imgLink){
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

		const ciphertext = encryptMessage(message, password);
		const ciphertextBits = stringToBits(ciphertext + '\0');

		let i = 0, j = 0;

		while(j < ciphertextBits.length){
			if(i >= data.length){
				alert("Image does not have enough opaque pixels");
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
    let password = '123';

	if(!imgLink){
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
		const message = decryptMessage(encryptedMessage, password);
		console.log(message);
		return message;
	}
}
