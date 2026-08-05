import { Service } from '@angular/core';

@Service()
export class UrlService {

    public wsUrl = "wss://"+window.location.host+"/ws";
}