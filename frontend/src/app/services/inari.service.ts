import { ElementRef, inject, isDevMode, Service, signal, ViewChild } from '@angular/core';
import { UrlService } from './url.service';
import { v4 as uuidv4 } from 'uuid';
import { Device } from 'mediasoup-client';
import { Consumer, Producer, RtpCapabilities, Transport } from 'mediasoup-client/types';
import NoSleep from 'nosleep.js';
import { ThreeService } from './three.service';
import { ErrorService } from './error.service';

enum PageState {
    WAIT_FOR_CODE = 0,
    LOGGING_IN = 1,
    CONNECTED = 2
}

type Client = { display_name: string, volume: number, producer_id: string, panner?: PannerNode, gain?: GainNode, consumer?: Consumer, stream?: MediaStream, audio?: HTMLAudioElement };

@Service()
export class InariService {
    // Injects
    private readonly urlService = inject(UrlService);
    private readonly threeService = inject(ThreeService);
    private readonly errorService = inject(ErrorService);

    // Internal
    public pageState = signal<PageState>(PageState.WAIT_FOR_CODE);
    public PageState = PageState;

    public isMuted = signal(false);
    public isForceMuted = signal(true);
    public isDeafened = signal(false);
    public displayName = signal("");

    private ws!: WebSocket;

    private device!: Device;
    private track!: MediaStreamTrack;
    private routerRtpCapabilities!: RtpCapabilities;
    private producer_transport!: Transport;
    private producer!: Producer;
    private consumers_transport!: Transport;
    private max_distance = 30;

    private pingInterval: any = null;

    public clients = signal<Client[]>([]);
    public clients_count = signal(1);

    private promises_ws: { uuid: string, res: any }[] = [];

    private noSleep: NoSleep = new NoSleep();
    private audioContext: AudioContext = new AudioContext();

    private setListenerPosition(x: number, y: number, z: number) {
        const listener = this.audioContext.listener;
        if (listener.positionX) {
            const t = this.audioContext.currentTime;
            listener.positionX.setValueAtTime(x, t);
            listener.positionY.setValueAtTime(y, t);
            listener.positionZ.setValueAtTime(z, t);
        } else {
            (listener as any).setPosition(x, y, z);
        }
    }

    async init(authCode: string, channel: String) {

        if (this.audioContext.state === 'closed') {
            this.audioContext = new AudioContext();
        }

        this.setListenerPosition(0, 0, 0);
        this.audioContext.resume().catch(() => { });

        return new Promise((res) => {
            document.body.style.animation = 'backgroundAnimationSpeed 1s linear infinite';
            this.pageState.set(this.PageState.LOGGING_IN);

            this.ws = new WebSocket(this.urlService.wsUrl);

            this.ws.onopen = async () => {
                console.log('Connected to inari ws !');
                const auth = await this.send_and_receive('auth', { auth_key: authCode, channel, is_connector: false });
                if (!auth.max_distance) return;
                this.max_distance = auth.max_distance;
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

                if (data.kind == 'error') {
                    this.errorService.setError(data.data.details);
                }

                let promise = this.promises_ws.find((v) => v.uuid == data.transaction_uuid);

                if (promise) {
                    promise.res(data.data);
                    this.promises_ws = this.promises_ws.filter((v) => v.uuid != data.transaction_uuid);
                    return;
                }

                if (data.kind == 'add_clients') {
                    data.data.clients.forEach((c: any) => this.add_client(c));
                } else if (data.kind == 'remove_client') {
                    const client = this.clients().find((c) => c.producer_id == data.data.id);
                    if (client) {
                        client.consumer?.close();
                        client.stream?.getTracks().forEach((t) => t.stop());
                        client.panner?.disconnect();
                        client.audio?.remove();
                    }
                    this.clients.update((v) => v.filter((c) => c.producer_id !== data.data.id));
                    this.clients_count.update(() => this.clients().length + 1);
                } else if (data.kind == 'sync_client_state') {
                    const { is_force_muted, is_muted, is_deafened, orientation, local_clients, display_name } = data.data;

                    this.displayName.set(display_name);
                    this.isForceMuted.set(is_force_muted);
                    if (this.isDeafened() != is_deafened) {
                        this.deafen(is_deafened);
                    }
                    if (this.isMuted() != is_muted) {
                        this.mute(is_muted);
                    }
                    const [x, y, z] = orientation;
                    const l = this.audioContext.listener;
                    const t = this.audioContext.currentTime;

                    if (l) {
                        l.forwardX.setTargetAtTime(x, t, 0.1);
                        l.forwardY.setTargetAtTime(y, t, 0.1);
                        l.forwardZ.setTargetAtTime(z, t, 0.1);
                    }

                    this.threeService.updateRotation(x, y, z);
                    local_clients.forEach((c: { display_name: string, position: number[], producer_id: string }) => {
                        const clients = this.clients();
                        const client = clients.find((c1) => c1.producer_id == c.producer_id);
                        const now = this.audioContext.currentTime;
                        if (client) {
                            client.display_name = c.display_name;
                            client.panner?.positionX.setTargetAtTime(c.position[0], now, 0.1);
                            client.panner?.positionY.setTargetAtTime(c.position[1], now, 0.1);
                            client.panner?.positionZ.setTargetAtTime(c.position[2], now, 0.1);

                            client.gain?.gain.setTargetAtTime(client.volume / 100, now, 0.1);
                        }
                        this.clients.set(clients.map((c) => c));
                    });
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

    async add_client(clientState: { producer_id: string, display_name: string }) {
        if (!this.clients().some((v) => v.producer_id == clientState.producer_id)) {
            const client: Client = { volume: 100, producer_id: clientState.producer_id, display_name: clientState.display_name };
            this.clients.update((v) => [...v, client]);
            this.clients_count.update(() => this.clients().length + 1);

            let value: any = await this.send_and_receive('consume', { producer_id: clientState.producer_id, rtp_capabilities: this.device.recvRtpCapabilities });

            if (value) {
                let { consumer_id, kind, rtp_parameters } = value;

                const consumer = await this.consumers_transport.consume({ id: consumer_id, producerId: clientState.producer_id, kind, rtpParameters: rtp_parameters });
                if (this.isDeafened()) {
                    consumer.pause();
                } else {
                    consumer.resume();
                }
                client.consumer = consumer;

                const stream = new MediaStream();
                stream.addTrack(consumer.track);

                const audio = new Audio();
                audio.muted = true;
                audio.srcObject = stream;

                client.audio = audio;
                client.stream = stream;

                const panner = this.audioContext.createPanner();
                panner.distanceModel = 'linear';
                panner.maxDistance = this.max_distance;
                panner.coneInnerAngle = 360;
                panner.rolloffFactor = 1;
                panner.refDistance = 1;

                const gain = this.audioContext.createGain();
                gain.gain.value = 1;

                client.panner = panner;
                client.gain = gain;
                let source = this.audioContext.createMediaStreamSource(stream);
                source.connect(gain)
                gain.connect(panner);
                panner.connect(this.audioContext.destination);

                audio.play().catch(() => { }); // For Chrome Bug
            }
        }
    }

    async send_and_receive(kind: string, data: any): Promise<any> {
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

            const clients_data: any = await this.send_and_receive('fetch_clients', {});
            clients_data.clients.forEach((c: any) => this.add_client(c));
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

        const media = await navigator.mediaDevices.getUserMedia({ audio: true }).catch(() => {
            this.errorService.setError('You must enable your microphone to use Inari!');
            this.disconnect();
        });

        if (!media) return;
        this.pageState.set(PageState.CONNECTED);
        document.body.style.animation = 'backgroundAnimation 20s linear infinite';
        this.track = media.getTracks()[0];
        this.producer = await this.producer_transport.produce({ track: this.track });
        this.producer.resume();
        this.noSleep.enable();
        this.threeService.show();
    }

    public async mute(value: boolean|null = null) {

        if (this.isForceMuted()) {
            return;
        }

        if (value == null) {
            value = !this.isMuted();
        }

        if (value) {
            this.producer.pause();
        } else {
            this.producer.resume();
        }

        this.send('sync_client_state', { is_muted: value, is_deafened: this.isDeafened() });
        this.isMuted.set(value);
    }

    public async deafen(value: boolean|null = null) {

        if (value == null) {
            value = !this.isDeafened();
        }

        if (value) {
            this.clients().forEach((v) => v.consumer?.pause());
        } else {
            this.clients().forEach((v) => v.consumer?.resume());
        }
        this.send('sync_client_state', { is_muted: this.isMuted(), is_deafened: value });
        this.isDeafened.set(value);
    }

    public async changeClientVolume(producer_id: string, value: string) {
        const clients = this.clients();
        const client = clients.find((c) => c.producer_id == producer_id);
        if (client) {
            client.volume = parseInt(value);
            this.clients.set(clients.map((c) => c));
        }
    }

    public async disconnect() {
        this.ws.close();
        document.body.style.animation = 'backgroundAnimation 20s linear infinite';
        if (this.pingInterval) {
            clearInterval(this.pingInterval);
        }
        this.promises_ws.forEach((v) => v.res(null));
        this.promises_ws = [];
        this.clients().forEach((c) => {
            c.consumer?.close();
            c.stream?.getTracks().forEach((t) => t.stop());
            c.panner?.disconnect();
            c.audio?.remove();
        });
        this.audioContext.close();
        this.clients.set([]);
        this.producer?.close();
        this.track?.stop();
        this.producer_transport?.close();
        this.consumers_transport?.close();
        this.threeService.destroy();
        this.pageState.set(PageState.WAIT_FOR_CODE);
        this.noSleep.disable();
    }
}
