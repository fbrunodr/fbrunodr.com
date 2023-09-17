const dropArea = document.getElementById("drop-area");
const inputFile = document.getElementById("input-file");
const imageView = document.getElementById("img-view");
let imageViewHeight = 0;

inputFile.addEventListener("change", uploadImage);

function uploadImage(){
	if(imageViewHeight === 0){
		imageViewHeight = imageView.clientHeight;
		imageView.style.height = imageViewHeight;
	}
	let imgLink = URL.createObjectURL(inputFile.files[0]);
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

