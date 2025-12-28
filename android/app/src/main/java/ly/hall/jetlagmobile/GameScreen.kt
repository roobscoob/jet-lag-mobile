package ly.hall.jetlagmobile

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import ly.hall.jetlagmobile.ui.theme.JetLagMobileTheme
import org.maplibre.android.MapLibre
import org.maplibre.android.camera.CameraPosition
import org.maplibre.android.geometry.LatLng
import org.maplibre.android.maps.MapLibreMap
import org.maplibre.android.maps.MapLibreMapOptions
import org.maplibre.android.maps.MapView
import org.maplibre.android.maps.Style
import uniffi.jet_lag_mobile.MapState
import uniffi.jet_lag_mobile.ViewState

class GameScreen : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        MapLibre.getInstance(this)
        enableEdgeToEdge()
        setContent { JetLagMobileTheme { MapLibreMap(modifier = Modifier.fillMaxSize()) } }
    }
}

@Composable
fun MapLibreMap(modifier: Modifier = Modifier) {
    val context = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current
    val viewState = remember { ViewState() }
    var mapState by remember { mutableStateOf<MapState?>(null) }
    var map by remember { mutableStateOf<MapLibreMap?>(null) }

    LaunchedEffect(viewState) { mapState = viewState.getMapState() }

    LaunchedEffect(map, mapState) {
        val m = map ?: return@LaunchedEffect
        val ms = mapState ?: return@LaunchedEffect
        m.setStyle(Style.Builder().fromJson(ms.getStyle()))
    }

    val mapView = remember {
        val options =
                MapLibreMapOptions.createFromAttributes(context).apply {
                    compassEnabled(false)
                    // need attribution on a splash screen tho
                    attributionEnabled(false)
                    logoEnabled(false)
                    // Set initial camera to Central Park, NYC
                    camera(
                        CameraPosition.Builder()
                            .target(LatLng(40.7571418, -73.9805655))
                            .zoom(12.0)
                            .build()
                    )
                }

        MapView(context, options).apply { getMapAsync { map = it } }
    }

    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event ->
            when (event) {
                Lifecycle.Event.ON_START -> mapView.onStart()
                Lifecycle.Event.ON_RESUME -> mapView.onResume()
                Lifecycle.Event.ON_PAUSE -> mapView.onPause()
                Lifecycle.Event.ON_STOP -> mapView.onStop()
                Lifecycle.Event.ON_DESTROY -> mapView.onDestroy()
                else -> {}
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose {
            lifecycleOwner.lifecycle.removeObserver(observer)
            mapView.onDestroy()
            mapState?.destroy()
            viewState.destroy()
        }
    }

    AndroidView(factory = { mapView }, modifier = modifier)
}
