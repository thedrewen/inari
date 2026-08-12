import { Service } from '@angular/core';

@Service()
export class UrlService {

    public wsUrl = "ws"+(window.location.protocol == 'https:' ? 's' : '')+"://"+(window.location.host == 'localhost:4200' ? 'localhost:3030' : window.location.host)+"/ws";
}