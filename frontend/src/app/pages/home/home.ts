import { Component, isDevMode, OnDestroy, OnInit, signal } from '@angular/core';
import { InariService } from '../../services/inari.service';
import { NavBar } from '../../components/nav-bar/nav-bar';
import { Footer } from '../../components/footer/footer';
import { ActivatedRoute } from '@angular/router';
import { Meta } from '@angular/platform-browser';


@Component({
    selector: 'app-home',
    imports: [NavBar, Footer],
    templateUrl: './home.html',
    styleUrl: './home.scss'
})
export class Home implements OnInit, OnDestroy {

    public settingsOpen = signal(false);

    constructor (
        public readonly is: InariService,
        private readonly activatedRoute: ActivatedRoute,
        private readonly meta: Meta
    ) {
        const query: any = this.activatedRoute.snapshot.queryParams;
        if (query.channel && query.code) {
            this.channel.set(query.channel);
            this.authCode.set(query.code);
            this.login();
        } else {
            if (!isDevMode()) {
                const default_channel = meta.getTag('name="default-channel"');
                this.channel.set(default_channel?.content ?? 'default');
            }
        }
    }

    ngOnInit(): void {}
    ngOnDestroy(): void {
        this.is.disconnect();
    }

    public onConnection = signal(false);

    async login() {
        this.onConnection.set(true);
        await this.is.init(this.authCode(), this.channel());
        this.onConnection.set(false);
    }

    public authCode = signal('');
    public channel = signal('default');

    updateAuthCode(value: string) {
        this.authCode.set(value);
    }

    updateChannel(value: string) {
        this.channel.set(value);
    }

    toggleSettings() {
        this.settingsOpen.update((v) => !v);
    }
}