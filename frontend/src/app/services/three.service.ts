import { Service } from '@angular/core';
import * as THREE from 'three';
import { STLLoader } from 'three/examples/jsm/loaders/STLLoader.js';

@Service()
export class ThreeService {

    private scene = new THREE.Scene();
    private camera : THREE.PerspectiveCamera | null = null; 
    private renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
    private head : THREE.Mesh | null = null;

    private loader = new STLLoader();

    public isDestroy = true;

    private targetQuat = new THREE.Quaternion();
    private clock = new THREE.Timer();
    private readonly SMOOTHING = 10;

    private targetDiv: HTMLDivElement | null = null;

    public async show() {
        this.isDestroy = false;
        this.targetDiv = document.getElementById('headcursor') as HTMLDivElement;
        this.targetDiv.appendChild(this.renderer.domElement);
        if (!this.camera) {
            this.camera = new THREE.PerspectiveCamera( 50, this.targetDiv.clientWidth / this.targetDiv.clientHeight, 0.1, 1000 );
        }
        this.renderer.setSize( this.targetDiv.clientWidth, this.targetDiv.clientHeight );

        if (!this.head) {
            const geometry = await this.loader.loadAsync('/assets/head.stl');
            geometry.rotateY(Math.PI / 2);
            geometry.center();
            const material = new THREE.MeshStandardMaterial( {
                color: 0xffffff,
                flatShading: true,
                polygonOffset: true,
                polygonOffsetFactor: 1,
                polygonOffsetUnits: 1,
                transparent: true,
                opacity: 0.9,
                side: THREE.DoubleSide,
                depthWrite: false
            } );
            this.head = new THREE.Mesh( geometry, material );

            const box = new THREE.Box3().setFromObject(this.head);
            const center = new THREE.Vector3();
            box.getCenter(center);
            this.head.geometry.translate(-center.x, -center.y, -center.z);

            if (this.targetDiv.clientWidth < 750) {
                this.head.scale.x = 0.5;
                this.head.scale.y = 0.5;
                this.head.scale.z = 0.5; 
            } else {
                this.head.scale.x = 0.8;
                this.head.scale.y = 0.8;
                this.head.scale.z = 0.8;    
            }

            const light = new THREE.DirectionalLight( 0xffffff, 3 );
            light.position.set( 10, 20, 30 );
            this.scene.add(light);

            this.scene.add(this.head);
            this.updateRotation(-0.75, 0, 0.7853982, true);
        }

        this.camera.position.z = 50;
        this.renderer.setClearColor( 0x000000, 0 );
        this.renderer.setAnimationLoop(() => this.animate());
    }

    private euler = new THREE.Euler(0, 0, 0, 'YXZ');
    public updateRotation(dx: number, dy: number, dz: number, snap: boolean = false) {
        if (!this.head) return;

        const yaw = Math.atan2(dx, dz);
        const pitch = -Math.asin(THREE.MathUtils.clamp(dy, -1, 1));

        this.euler.set(pitch, yaw, 0.7853982, 'YXZ');
        this.targetQuat.setFromEuler(this.euler);
        if (snap) this.head.quaternion.copy(this.targetQuat);
    }

    private async animate() {

        if (this.targetDiv) {
            this.renderer.setSize( this.targetDiv.clientWidth, this.targetDiv.clientHeight );
            if (this.camera) {
                this.camera.aspect = this.targetDiv.clientWidth / this.targetDiv.clientHeight;
                this.camera.updateProjectionMatrix();
            }

            if (this.head) {
                if (this.targetDiv.clientWidth < 750) {
                    this.head.scale.x = 0.5;
                    this.head.scale.y = 0.5;
                    this.head.scale.z = 0.5; 
                } else {
                    this.head.scale.x = 0.8;
                    this.head.scale.y = 0.8;
                    this.head.scale.z = 0.8;    
                }
            }
        }

        this.clock.update();
        const dt = this.clock.getDelta();
        if (this.head) {
            const t = 1 - Math.exp(-this.SMOOTHING * dt);
            this.head.quaternion.slerp(this.targetQuat, t);
        }
        if (this.camera) {
            this.renderer.render(this.scene, this.camera);
        }
    }

    public destroy() {
        this.isDestroy = true;
    }
}