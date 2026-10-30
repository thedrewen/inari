import { Component, OnDestroy, OnInit, signal } from '@angular/core';
import { RouterOutlet } from '@angular/router';

@Component({
    selector: 'app-root',
    imports: [RouterOutlet],
    templateUrl: './app.html',
    styleUrl: './app.scss'
})
export class App implements OnInit {
    isFirefox = signal(false);
    
    ngOnInit(): void {
        // @ts-expect-error
        this.isFirefox.set(typeof InstallTrigger !== 'undefined');
    }
}