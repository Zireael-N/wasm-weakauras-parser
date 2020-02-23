init('./wasm_weakauras_parser_bg.wasm').then(() => self.postMessage({
    message: 'initialized',
    data: null,
}));

self.onmessage = ({ data }) => {
    try {
        const parsed = parse(data);
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
};
