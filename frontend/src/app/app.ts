import { Component, OnDestroy, OnInit, signal } from '@angular/core';
import { InariService } from './services/inari.service';


@Component({
    selector: 'app-root',
    imports: [],
    templateUrl: './app.html',
    styleUrl: './app.scss'
})
export class App implements OnInit, OnDestroy {

    constructor (
        public readonly is: InariService
    ) { }

    ngOnInit(): void {}
    ngOnDestroy(): void {
        this.is.disconnect();
    }

    public onConnection = signal(false);

    async login() {
        this.onConnection.set(true);
        await this.is.init(this.authCode());
        this.onConnection.set(false);
    }

    public authCode = signal('');

    updateAuthCode(value: string) {
        this.authCode.set(value);
    }
}