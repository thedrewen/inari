import { Component, inject, OnDestroy, OnInit, signal } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { ErrorService } from './services/error.service';

@Component({
    selector: 'app-root',
    imports: [RouterOutlet],
    templateUrl: './app.html',
    styleUrl: './app.scss'
})
export class App implements OnInit {
    isFirefox = signal(false);
    
    constructor (
        public readonly er: ErrorService
    ) { }
    
    ngOnInit(): void {
        // @ts-expect-error
        this.isFirefox.set(typeof InstallTrigger !== 'undefined');
    }
}