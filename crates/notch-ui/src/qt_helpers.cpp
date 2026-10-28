#include "qt_helpers.h"

#include <QGuiApplication>
#include <QObject>
#include <QRect>
#include <QRegion>
#include <QScreen>
#include <QVariant>
#include <QWindow>

namespace {

QWindow *to_window(std::uint64_t win_ptr)
{
    return reinterpret_cast<QWindow *>(win_ptr);
}

// QQuickWindow hereda de QWindow, así que basta QtGui para el cast.
QWindow *find_host_window(QObject *obj)
{
    for (QObject *o = obj; o != nullptr; o = o->parent()) {
        if (auto *w = qobject_cast<QWindow *>(o)) {
            return w;
        }
        const QVariant v = o->property("window");
        if (!v.isValid()) {
            continue;
        }
        QObject *candidate = v.value<QObject *>();
        if (candidate == nullptr && (v.metaType().flags() & QMetaType::PointerToQObject)) {
            candidate = *static_cast<QObject *const *>(v.constData());
        }
        if (auto *w = qobject_cast<QWindow *>(candidate)) {
            return w;
        }
    }
    return nullptr;
}

} // namespace

std::uint64_t notch_find_host_window(std::uint64_t bridge_ptr)
{
    auto *bridge = reinterpret_cast<QObject *>(bridge_ptr);
    QWindow *win = find_host_window(bridge);
    return reinterpret_cast<std::uint64_t>(win);
}

void notch_configure_fallback(std::uint64_t win_ptr)
{
    QWindow *win = to_window(win_ptr);
    if (win == nullptr) {
        return;
    }
    win->setFlags(Qt::Window | Qt::FramelessWindowHint | Qt::WindowStaysOnTopHint
                  | Qt::WindowDoesNotAcceptFocus);
}

void notch_set_geometry(std::uint64_t win_ptr, double x, double y, double w, double h)
{
    QWindow *win = to_window(win_ptr);
    if (win == nullptr) {
        return;
    }
    win->setGeometry(QRect(static_cast<int>(x), static_cast<int>(y),
                           static_cast<int>(w), static_cast<int>(h)));
}

void notch_apply_mask(std::uint64_t win_ptr, double x, double y, double w, double h)
{
    QWindow *win = to_window(win_ptr);
    if (win == nullptr) {
        return;
    }
    win->setMask(QRegion(QRect(static_cast<int>(x), static_cast<int>(y),
                               static_cast<int>(w), static_cast<int>(h))));
}

void notch_clear_mask(std::uint64_t win_ptr)
{
    QWindow *win = to_window(win_ptr);
    if (win == nullptr) {
        return;
    }
    // QWindow (Qt 6.4) no tiene clearMask(); una región vacía elimina la máscara.
    win->setMask(QRegion());
}

bool notch_primary_screen(double &x, double &y, double &w, double &h, double &dpr)
{
    const QScreen *screen = QGuiApplication::primaryScreen();
    if (screen == nullptr) {
        return false;
    }
    const QRect g = screen->geometry();
    x = g.x();
    y = g.y();
    w = g.width();
    h = g.height();
    dpr = screen->devicePixelRatio();
    return true;
}
