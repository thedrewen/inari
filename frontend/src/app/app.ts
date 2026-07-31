import { Component, signal } from '@angular/core';

enum PageState {
    WAIT_FOR_CODE = 0
}

@Component({
    selector: 'app-root',
    imports: [],
    templateUrl: './app.html',
    styleUrl: './app.scss'
})
export class App {
    public pageState = signal<PageState>(PageState.WAIT_FOR_CODE);
    public PageState = PageState;

    constructor () { }
}