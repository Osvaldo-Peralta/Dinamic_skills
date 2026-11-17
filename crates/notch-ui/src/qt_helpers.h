#pragma once

#include <cstdint>

// Operaciones sobre QWindow recibidos como punteros crudos (u64) para no
// exponer tipos de Qt en las firmas del bridge. Toda la conversión insegura
// queda contenida en este archivo.

// Busca el QWindow que aloja a un QObject (subiendo por antepasados y por la
// propiedad "window" de los QQuickItem). Devuelve 0 si no lo encuentra.
std::uint64_t notch_find_host_window(std::uint64_t bridge_ptr);
// Sin bordes, siempre encima, sin aceptar foco (backend de respaldo, spec §7).
void notch_configure_fallback(std::uint64_t win_ptr);
// Geometría global de la ventana.
void notch_set_geometry(std::uint64_t win_ptr, double x, double y, double w, double h);
// Región interactiva: máscara rectangular en coordenadas de ventana.
void notch_apply_mask(std::uint64_t win_ptr, double x, double y, double w, double h);
// Quita la máscara (toda la ventana interactiva).
void notch_clear_mask(std::uint64_t win_ptr);
// Geometría y relación de píxeles del monitor principal; false si no hay.
bool notch_primary_screen(double &x, double &y, double &w, double &h, double &dpr);
