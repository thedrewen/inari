import { Component, input } from '@angular/core';
import { InariService } from '../../services/inari.service';
import { RouterLink } from '@angular/router';

@Component({
  imports: [RouterLink],
  selector: 'app-nav-bar',
  styleUrl: './nav-bar.scss',
  templateUrl: './nav-bar.html',
})
export class NavBar {
  channel = input<string | null>();
  clients_count = input<number | null>();
}
