import { createPlatform, inject, Service, signal } from '@angular/core';
import { UrlService } from './url.service';
import { v4 as uuidv4 } from 'uuid';
import { Device } from 'mediasoup-client';
import { Consumer, Producer, RtpCapabilities, Transport } from 'mediasoup-client/types';
import NoSleep from 'nosleep.js';

enum PageState {
    WAIT_FOR_CODE = 0,
    LOGGING_IN = 1,
    CONNECTED = 2
}

type Client = { producer_id: string, consumer?: Consumer, stream?: MediaStream, audio?: HTMLAudioElement };

@Service()
export class InariService {

    // Injects
    private readonly urlService = inject(UrlService);

    // Internal
    public pageState = signal<PageState>(PageState.WAIT_FOR_CODE);
    public PageState = PageState;

    public isMute = signal(false);
    public isDaefen = signal(false);

    private ws!: WebSocket;

    private device!: Device;
    private track!: MediaStreamTrack;
    private routerRtpCapabilities!: RtpCapabilities;
    private producer_transport!: Transport;
    private producer!: Producer;
    private consumers_transport!: Transport;

    private pingInterval: any = null;

    private clients: Client[] = [];
    public clients_count = signal(1);

    private promises_ws: { uuid: string, res: any }[] = [];

    private noSleep: NoSleep = new NoSleep();

    async init(authCode: string, channel: String) {
        return new Promise((res) => {
            document.body.style.animation = 'backgroundAnimationSpeed 1s linear infinite';
            this.pageState.set(this.PageState.LOGGING_IN);

            this.ws = new WebSocket(this.urlService.wsUrl);

            this.ws.onopen = async () => {
                console.log('Connected to inari ws !');
                await this.send('auth', { auth_key: authCode, channel, is_server: false });
                this.send_and_receive('rtp_capabilities', {}).then((r: any) => this.init_mediasoup(r));

                this.device = new Device();
                this.pingInterval = setInterval(() => {
                    this.send('ping', {});
                }, 20000);
                res(null);
            }

            this.ws.onclose = () => {
                this.disconnect();
                res(null);
            }

            this.ws.onmessage = async (message) => {
                const data = JSON.parse(message.data);

                let promise = this.promises_ws.find((v) => v.uuid == data.transaction_uuid);

                if (promise) {
                    promise.res(data.data);
                    this.promises_ws = this.promises_ws.filter((v) => v.uuid != data.transaction_uuid);
                    return;
                }

                if (data.kind == 'add_clients') {
                    data.data.ids.forEach((c: string) => this.add_client(c));
                } else if (data.kind == 'remove_client') {
                    const client = this.clients.find((c) => c.producer_id == data.data.id);
                    if (client) {
                        client.consumer?.close();
                        client.stream?.getTracks().forEach((t) => t.stop());
                        client.audio?.remove();
                    }
                    this.clients = this.clients.filter((c) => c.producer_id !== data.data.id);
                    this.clients_count.update(() => this.clients.length + 1);
                }
            }
        });
    }

    async send(kind: string, data: any) {
        try {
            let uuid = uuidv4()
            this.ws.send(JSON.stringify({ kind, data, transaction_uuid: uuid }));
        } catch (error) {
            await this.disconnect();
        }
    }

    async add_client(producer_id: string) {
        if (!this.clients.some((v) => v.producer_id == producer_id)) {
            const client: Client = { producer_id };
            this.clients.push(client);
            this.clients_count.update(() => this.clients.length + 1);

            let value: any = await this.send_and_receive('consume', { producer_id: producer_id, rtp_capabilities: this.device.recvRtpCapabilities });

            if (value) {
                let { consumer_id, kind, rtp_parameters } = value;

                const consumer = await this.consumers_transport.consume({ id: consumer_id, producerId: producer_id, kind, rtpParameters: rtp_parameters });
                if (this.isDaefen()) {
                    consumer.pause();
                } else {
                    consumer.resume();
                }
                client.consumer = consumer;

                const stream = new MediaStream();
                stream.addTrack(consumer.track);

                const audio = new Audio();
                audio.srcObject = stream;
                audio.play();

                client.audio = audio;
                client.stream = stream;
            }
        }
    }

    async send_and_receive(kind: string, data: any) {
        let uuid = uuidv4()
        try {
            this.ws.send(JSON.stringify({ kind, data, transaction_uuid: uuid }));
        } catch (error) {
            await this.disconnect();
            return null;
        }

        return new Promise((res) => {
            this.promises_ws.push({ uuid, res });
        });
    }

    async init_mediasoup(r: any) {
        this.routerRtpCapabilities = r.rtp_capabilities;
        await this.device.load({ routerRtpCapabilities: this.routerRtpCapabilities });
        const result: any = await this.send_and_receive('create_transport', { is_producer: true });


        this.send_and_receive('create_transport', { is_producer: false }).then(async (result: any) => {
            const { id, dtls_parameters, ice_candidates, ice_parameters } = result;

            this.consumers_transport = this.device.createRecvTransport({ id, iceParameters: ice_parameters, dtlsParameters: dtls_parameters, iceCandidates: ice_candidates });
            this.consumers_transport.on('connect', async ({ dtlsParameters }, callback, errback) => {
                let result = await this.send_and_receive('connect_transport', { is_producer: false, dtls_parameters: dtlsParameters });
                if (result) {
                    callback();
                } else {
                    errback(Error());
                }
            });

            const clients: any = await this.send_and_receive('fetch_clients', {});
            clients.ids.forEach((c: string) => this.add_client(c));
        });


        const { id, dtls_parameters, ice_candidates, ice_parameters } = result;
        this.producer_transport = this.device.createSendTransport({ id, iceParameters: ice_parameters, dtlsParameters: dtls_parameters, iceCandidates: ice_candidates });

        this.producer_transport.on('connect', async ({ dtlsParameters }, callback, errback) => {
            let result = await this.send_and_receive('connect_transport', { is_producer: true, dtls_parameters: dtlsParameters });
            if (result) {
                callback();
            } else {
                errback(Error());
            }
        });

        this.producer_transport.on('produce', async ({ kind, rtpParameters }, callback, errback) => {
            let result: any = await this.send_and_receive('produce', { kind, rtp_parameters: rtpParameters });
            if (result) {
                callback(result.id);
            } else {
                errback(Error());
            }
        });

        const media = await navigator.mediaDevices.getUserMedia({ audio: true }).catch(() => this.disconnect());

        if (!media) return;
        this.pageState.set(PageState.CONNECTED);
        document.body.style.animation = 'backgroundAnimation 20s linear infinite';
        this.track = media.getTracks()[0];
        this.producer = await this.producer_transport.produce({ track: this.track });
        this.producer.resume();
        this.noSleep.enable();
    }

    public async mute() {
        if (this.isMute()) {
            this.producer.resume();
        } else {
            this.producer.pause();
        }

        this.isMute.update((v) => !v);
    }

    public async daefen() {
        if (this.isDaefen()) {
            this.clients.forEach((v) => v.consumer?.resume());
        } else {
            this.clients.forEach((v) => v.consumer?.pause());
        }

        this.isDaefen.update((v) => !v);
    }

    public async disconnect() {
        this.ws.close();
        document.body.style.animation = 'backgroundAnimation 20s linear infinite';
        if (this.pingInterval) {
            clearInterval(this.pingInterval);
        }
        this.promises_ws.forEach((v) => v.res(null));
        this.promises_ws = [];
        this.clients.forEach((c) => {
            c.consumer?.close();
            c.stream?.getTracks().forEach((t) => t.stop());
            c.audio?.remove();
        });
        this.clients = [];
        this.producer?.close();
        this.track?.stop();
        this.producer_transport?.close();
        this.consumers_transport?.close();
        this.pageState.set(PageState.WAIT_FOR_CODE);
        this.noSleep.disable();
    }
}
