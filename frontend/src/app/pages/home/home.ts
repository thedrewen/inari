import { Component, OnDestroy, OnInit, signal } from '@angular/core';
import { InariService } from '../../services/inari.service';
import { NavBar } from '../../components/nav-bar/nav-bar';
import { Footer } from '../../components/footer/footer';


@Component({
    selector: 'app-home',
    imports: [NavBar, Footer],
    templateUrl: './home.html',
    styleUrl: './home.scss'
})
export class Home implements OnInit, OnDestroy {

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
}