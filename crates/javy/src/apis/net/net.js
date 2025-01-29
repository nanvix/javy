(function () {
    const __net_socket_accept = globalThis.__net_socket_accept;
    const __net_socket_read = globalThis.__net_socket_read;
    const __net_socket_write = globalThis.__net_socket_write;

    globalThis.net.socket = {
        accept() {
            return __net_socket_accept();
        },
        read(sockfd, buffer, byteOffset, byteLength) {
            if (!(buffer instanceof Uint8Array)) {
                throw new TypeError("Data must be a Uint8Array");
            }
            return __net_socket_read(
                sockfd,
                buffer.buffer,
                byteOffset,
                byteLength
            );
        },
        write(sockfd, buffer, byteOffset, byteLength) {
            if (!(buffer instanceof Uint8Array)) {
                throw new TypeError("Data must be a Uint8Array");
            }
            return __net_socket_write(
                sockfd,
                buffer.buffer,
                byteOffset,
                byteLength
            );
        },
    };

    Reflect.deleteProperty(globalThis, "__net_socket_accept");
    Reflect.deleteProperty(globalThis, "__net_socket_read");
    Reflect.deleteProperty(globalThis, "__net_socket_write");
})();
