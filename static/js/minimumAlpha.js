function arraysAreEqual(arr1, arr2) {
    return arr1.length === arr2.length && arr1.every((value, index) => value === arr2[index]);
}

const is_good = (alpha) => {
    const can = document.createElement('canvas');
    const ctx = can.getContext('2d');
    can.width = 1;
    can.height = 1;
    const img = ctx.createImageData(1, 1);
    for(let i = 0; i <= 255; i++){
        img.data[0] = i;
        img.data[1] = i;
        img.data[2] = i;
        img.data[3] = alpha;
        ctx.putImageData(img, 0, 0);
        if(!arraysAreEqual(ctx.getImageData(0, 0, 1, 1).data, img.data)){
            console.log(`${alpha} alpha failed at color = ${i}`);
            return false;
        }
    }
        
    return true;
}

for(let alpha = 0; alpha <= 255; alpha++)
    if(is_good(alpha))
        console.log(alpha);
