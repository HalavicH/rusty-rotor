## TODO:
- [x] Figure out input from the controller
- [x] Setup drone entity (cube)
- [x] Wire inputs from the controller to the drone to control its axis movements
- [x] Visualize stick positions
- [ ] Import drone model
- [ ] Add initial gravity simulation
- [ ] Add thrust system for quadrotor movement
- [ ] Implement basic collision detection with objects

### Milestone 1 - Implement physics using rapier3d or avian
Evaluate both libraries and choose one for the project.
What physics engine should do:
- simulate gravity
- simulate rigid body dynamics
    - simulate collisions
- simulate forces (thrust, drag, wind resistance)
- simulate fluid dynamics (for wind)
DoD:
- basic demo with falling cubes, different shapes, weights
- verbal info of how each physics feature works

### Milestone 2 – Implement level/scene loading
Will unlock the ability to load different levels to test features in isolation.
features:
- can play/pause the simulation, restart the level
- persistence of the level state
- level loading from a file
Resources:
- level assets as entities with their components and component state
- some library to load/save level state
Load process:
- load static entities from gltf file
- load component state from a file
- patch entities with their state
- spawn dynamic entities from the file with their component states
DoD:
- load initial scene with static entities, spawn dynamic entities
- change camera position/entity position, save it to a file
- load the scene from a file with persisted changes
