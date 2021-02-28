init('./wasm_weakauras_parser_bg.wasm').then(() => self.postMessage({
    message: 'initialized',
    data: null,
}));

self.onmessage = ({ data: { action, data } }) => {
    switch(action) {
        case 'decode': {
            try {
                const parsed = decode(data);
                self.postMessage({
                    message: 'completed',
                    data: parsed,
                });
            } catch(e) {
                self.postMessage({
                    message: 'failure',
                    data: e.message,
                });
            }
            break;
        }
        case 'encode': {
            try {
                const string = encode(JSON.stringify(data));
                console.log(string);
            } catch(e) {
                console.error(e);
            }
            break;
        }
        default:
            console.error('Unknown action');
    }
};
