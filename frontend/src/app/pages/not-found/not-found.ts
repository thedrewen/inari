import { Component } from '@angular/core';
import { NavBar } from '../../components/nav-bar/nav-bar';

@Component({
  imports: [NavBar],
  selector: 'app-not-found',
  styleUrl: './not-found.scss',
  templateUrl: './not-found.html',
})
export class NotFound {}