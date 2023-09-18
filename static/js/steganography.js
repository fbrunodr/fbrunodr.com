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
  // Initialize an empty array to store the bits
  const bits = [];

  // Iterate through each character in the input string
  for (let i = 0; i < inputString.length; i++) {
    // Get the character code of the current character
    const charCode = inputString.charCodeAt(i);

    // Convert the character code to its binary representation (8 bits)
    const binaryString = charCode.toString(2);

    // Pad the binary representation to ensure it's 8 bits long
    const paddedBinary = binaryString.padStart(8, '0');

    // Split the padded binary string into individual bits and add them to the bits array
    for (let j = 0; j < paddedBinary.length; j++) {
      bits.push(parseInt(paddedBinary[j]));
    }
  }

  return bits;
}


function bitsToString(bitsArray) {
  // Ensure the input array has a multiple of 8 bits (8 bits make 1 byte)
  if (bitsArray.length % 8 !== 0) {
    throw new Error("Input does not have a valid binary representation");
  }

  // Initialize an empty string to store the reconstructed string
  let reconstructedString = "";

  // Process the binary array in 8-bit chunks
  for (let i = 0; i < bitsArray.length; i += 8) {
    // Extract the next 8 bits
    const byteBits = bitsArray.slice(i, i + 8);

    // Convert the binary bits to a decimal value
    const decimalValue = parseInt(byteBits.join(""), 2);

    // Convert the decimal value to a character and add it to the string
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

		for(let i = 0, j = 0; i < ciphertextBits.length; i++, j++){
			if(j % 4 == 3)
				j++;
			data[j] = (data[j] & 254) | ciphertextBits[i];
		}

		imageData.data = data;
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

		while(zerosInRow != 8){
			if(i % 4 == 3)
				i++;
			const bit = data[i] & 1;
			bits.push(bit);
			if(bit)
				zerosInRow = 0;
			else
				zerosInRow++;
			i++;
		}
		
		canvas.remove();
		const encryptedMessage = bitsToString(bits).slice(0,-1);
		const message = decryptMessage(encryptedMessage, password);
		console.log(message);
		return message;
	}
}

