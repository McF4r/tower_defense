# Development Note

## Bevy Engine

- Bevy follows the idea of ECS Design, which means every object on the scene has a "Entity ID".
- With Entity ID, you can insert "Components" to that object. Components include "Lifetime", "Bullet", "GameAssets", "Tower", "Target", "Health", which are all properties a object have. And commands are the ways to manipulate Components.
- Systems are actual codes to manipulate Components behavious, control the entire game world.
