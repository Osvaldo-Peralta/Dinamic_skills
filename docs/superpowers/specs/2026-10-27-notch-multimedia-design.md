# Notch multimedia para escritorio: especificación de diseño (v1)

Fecha: 2026-10-27
Estado: borrador pendiente de revisión

## 1. Propósito

Una isla dinámica (notch) para el escritorio, optimizada para consumir contenido. El núcleo es la reproducción multimedia. El proyecto nace para uso personal en CachyOS con Cinnamon y se publicará al público más adelante, por lo que la portabilidad entre entornos es un requisito de diseño.

## 2. Alcance

### Dentro de la v1
- Reproductor multimedia que ocupa todo el notch, con controles, portada y progreso.
- Vista ampliada al pasar el mouse.
- Funcionamiento en Linux (X11 y Wayland) y, como segundo objetivo, en Windows.

### Fuera de la v1 (se decide después de que la v1 funcione)
- Notificaciones del sistema y de redes sociales. El problema de ser el demonio de notificaciones quedó pospuesto; cualquier solución futura debe respetar el requisito de no obligar al usuario a modificar su sistema.
- Feeds y novedades (por ejemplo RSS).
- Herramientas y aplicaciones del día a día.
- Cambio manual entre reproductores.
- Varios monitores a la vez (la v1 usa solo el monitor principal).

## 3. Requisitos transversales

- **Sin terminal:** instalar y usar el notch debe funcionar con clics, sin comandos y sin alterar la configuración del escritorio.
- **Rendimiento:** consumo casi nulo en reposo. El rendimiento y la calidad visual tienen prioridad sobre el tamaño del paquete.
- **Portabilidad:** X11, Wayland (varios compositores) y Windows, mediante piezas intercambiables.

## 4. Tecnología

Rust para el núcleo y los adaptadores, con Qt6/QML para la interfaz. El puente entre ambos se haría con `cxx-qt`. La elección se justifica por las animaciones fluidas por GPU y la capacidad de personalización visual.

## 5. Arquitectura

Un solo proceso, organizado como workspace de Rust con módulos de fronteras claras.

| Módulo | Responsabilidad |
|---|---|
| `notch-core` | Estado del reproductor (pista, artista, portada, posición, reproduciendo o en pausa, capacidades), comandos (reproducir/pausar, siguiente, anterior, saltar a posición), máquina de estados del notch y regla de selección de reproductor. No depende de Qt ni del sistema operativo. Declara las interfaces de fuente de medios y de backend de ventana. |
| `notch-mpris` | Fuente de medios en Linux: MPRIS por D-Bus. Descubre reproductores, traduce su estado al núcleo y gestiona la descarga y el caché de portadas. |
| `notch-gsmtc` | Fuente de medios en Windows: controles multimedia globales del sistema. Misma interfaz que `notch-mpris`. |
| `notch-window-x11`, `notch-window-wayland`, `notch-window-fallback`, `notch-window-win32` | Backends de ventana intercambiables. |
| `notch-ui` | Interfaz en QML. Dibuja el estado y reporta eventos; no contiene lógica de negocio. |
| `notch-app` | Ensamblador: detecta el entorno al arrancar, elige backend y fuente de medios, y conecta las piezas. |

Flujo: un cambio en el reproductor llega por la fuente de medios al núcleo y la interfaz se redibuja. En sentido contrario, un evento de la interfaz pasa por el núcleo y sale como comando hacia el reproductor o como cambio de estado del notch.

## 6. Comportamiento del notch

### Estados
- **Sin reproductor:** el notch se oculta por completo y reaparece cuando algún reproductor empieza.
- **Reposo:** píldora compacta que ocupa todo el notch con portada en miniatura, título y artista (con desplazamiento si no caben), controles básicos (anterior, reproducir/pausar, siguiente) y barra de progreso fina.
- **Expandido (hover):** portada grande, título y artista completos, barra de progreso con salto de posición, controles ampliados y nombre del reproductor.

### Transiciones
- Retardo al entrar el mouse: 150 ms como valor inicial. Retardo al salir: 400 ms como valor inicial. Ambos son ajustables y se afinarán con pruebas reales.
- Con el notch quieto, las animaciones y el redibujado se detienen.

### Selección del reproductor activo
Manda el que está reproduciendo. Si hay varios reproduciendo, el que empezó más recientemente. Si ninguno reproduce, el último que lo hizo.

## 7. Ventana y entornos

### Elección del backend al arrancar
Automática, sin configuración del usuario:
- **X11:** ventana sin bordes, tipo panel, siempre encima, con la zona sensible al mouse limitada a la píldora.
- **Wayland con layer-shell** (KDE, Hyprland, Sway y similares): ventana de panel nativa.
- **Respaldo** (por ejemplo GNOME): ventana normal sin bordes y siempre encima. Limitación documentada: algunos entornos no garantizan que se mantenga por encima de todo.
- **Windows:** ventana siempre encima que no roba el foco, con zona sensible limitada a la píldora.
- Si el backend preferido falla al iniciar, se cae al siguiente y se registra en el log.

### Posición
Centrado en el borde superior del monitor principal. Se reposiciona si cambia la resolución o se conectan o desconectan pantallas.

## 8. Manejo de errores y casos límite

- Reproductor que se cierra o falla a mitad: el notch se oculta sin errores y vuelve cuando aparezca otro.
- Metadatos incompletos: imagen de relleno si no hay portada; en transmisiones en vivo la barra de progreso se oculta o queda indeterminada.
- Controles no soportados: se atenúan según las capacidades que declare el reproductor.
- Portada que no se descarga: descarga en segundo plano con tiempo límite, sin bloquear la interfaz.
- Los fallos se registran en un archivo de log. El usuario nunca ve errores técnicos.

## 9. Pruebas y criterios de éxito

### Pruebas por capa
- **Núcleo:** pruebas automáticas, sin pantalla, de la máquina de estados y de la regla de selección.
- **Fuentes de medios:** pruebas automáticas contra un reproductor simulado que cubra metadatos completos, incompletos, transmisión en vivo, cierre abrupto y varios reproductores.
- **Ventana e interfaz:** lista de verificación manual con Spotify, YouTube y Netflix reales, en el escritorio del autor (Linux) y en Windows. La cantidad de información que expone cada servicio (en particular Netflix) se confirma con la prueba real.
- **Rendimiento:** se mide el consumo en reposo (CPU y memoria). Los umbrales numéricos se fijan al cierre del Hito 1, con datos del prototipo, y quedan registrados en este documento antes de iniciar el Hito 2.

### La v1 está lista cuando
- Aparece al empezar a reproducir en Spotify, YouTube y Netflix, y se oculta al cerrarlos.
- Los controles funcionan donde el reproductor los permita.
- El hover abre y cierra la vista ampliada sin parpadeos.
- El consumo en reposo se mantiene bajo los umbrales fijados.
- Se instala y arranca sin abrir la terminal.

## 10. Distribución

- **Linux:** paquete Flatpak como vía principal, con los permisos de D-Bus para MPRIS declarados en el paquete. Inicio automático con la sesión, activable desde la propia app con un interruptor.
- **Windows:** instalador o ejecutable que se usa con doble clic.

## 11. Hitos

1. **Hito 1: prototipo de la ventana flotante** en Linux y en Windows. Valida que los backends intercambiables funcionan en la práctica (siempre encima, sin robar el foco, hover, esquinas). Al cerrarlo se fijan los umbrales de rendimiento.
2. **Hito 2: v1 completa en Linux** (fuente MPRIS, núcleo, interfaz, pruebas, Flatpak).
3. **Hito 3: v1 completa en Windows** (fuente de medios de Windows, instalador, pruebas).

## 12. Riesgos conocidos

- La ventana flotante es la parte más incierta; por eso es el primer hito.
- El puente entre Rust y Qt (`cxx-qt`) exige tener Qt y CMake en el entorno de compilación.
- Wayland no es uniforme: en GNOME el comportamiento de ventana siempre encima puede ser limitado.
- La información que exponen Spotify, YouTube y Netflix puede variar entre Linux y Windows.
